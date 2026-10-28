//! 单块下载：Range 请求 + 随机写到目标文件的指定偏移

use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::Mutex;

use super::super::mirror_gate;
use super::super::rate_limiter::RateLimiter;
use super::super::types::GlobalProgress;
use super::table::Segment;

/// 单块下载失败原因
#[derive(Debug)]
pub(crate) enum BlockError {
    /// 服务端对非首块返回 200：不支持 Range，这条源不能用于分块并行
    RangeUnsupported,
    Cancelled,
    Failed(String),
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::RangeUnsupported => write!(f, "服务端不支持 Range"),
            BlockError::Cancelled => write!(f, "下载已取消"),
            BlockError::Failed(e) => write!(f, "{}", e),
        }
    }
}

/// 下载一块并写到文件的对应偏移
///
/// 块级不做续传：块内失败就整块重下（2 MiB 重下成本远低于对齐偏移的复杂度）。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn download_block(
    client: &reqwest::Client,
    url: &str,
    path: &str,
    segment: Segment,
    rate_limiter: &Option<Arc<Mutex<RateLimiter>>>,
    file_progress: &Option<Arc<StdMutex<GlobalProgress>>>,
    source_bytes: &Arc<AtomicU64>,
    pause_flag: &Option<Arc<AtomicBool>>,
    cancel_flag: &Option<Arc<AtomicBool>>,
) -> Result<u64, BlockError> {
    let expected = segment.len();
    // 镜像闸门：许可持有到响应体读完，公共镜像的高并发 Range 才有节制
    let _permit = mirror_gate::acquire(url).await;
    let response = client
        .get(url)
        .header(
            reqwest::header::RANGE,
            format!("bytes={}-{}", segment.start, segment.end),
        )
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .timeout(Duration::from_secs(600))
        .send()
        .await
        .map_err(|e| BlockError::Failed(crate::http::request_error_msg(&e)))?;
    mirror_gate::observe(&response);
    let status = response.status();
    if status == reqwest::StatusCode::OK && segment.start > 0 {
        return Err(BlockError::RangeUnsupported);
    }
    if !status.is_success() {
        return Err(BlockError::Failed(format!("HTTP {}", status.as_u16())));
    }

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|e| BlockError::Failed(format!("打开目标文件失败: {}", e)))?;
    file.seek(SeekFrom::Start(segment.start))
        .map_err(|e| BlockError::Failed(format!("定位写入偏移失败: {}", e)))?;

    let mut downloaded = 0_u64;
    let mut stream = response.bytes_stream();
    loop {
        if cancel_requested(cancel_flag) {
            return Err(BlockError::Cancelled);
        }
        while pause_flag
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            if cancel_requested(cancel_flag) {
                return Err(BlockError::Cancelled);
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        let next = tokio::time::timeout(Duration::from_secs(20), stream.next()).await;
        let chunk = match next {
            Err(_) => return Err(BlockError::Failed("20s 无数据流动".to_string())),
            Ok(None) => break,
            Ok(Some(Err(e))) => return Err(BlockError::Failed(e.to_string())),
            Ok(Some(Ok(chunk))) => chunk,
        };
        let chunk_len = chunk.len() as u64;
        if downloaded + chunk_len > expected {
            return Err(BlockError::Failed(
                "服务端返回的数据超过请求区间".to_string(),
            ));
        }
        write_with_limit(
            &mut file,
            &chunk,
            rate_limiter,
            &mut downloaded,
            cancel_flag,
        )
        .await?;
        source_bytes.fetch_add(chunk_len, Ordering::Relaxed);
        if let Some(progress) = file_progress {
            let mut guard = progress
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.downloaded_bytes = guard.downloaded_bytes.saturating_add(chunk_len);
        }
    }

    if downloaded != expected {
        return Err(BlockError::Failed(format!(
            "块大小不符：实际 {} 字节，期望 {}",
            downloaded, expected
        )));
    }
    Ok(downloaded)
}

fn cancel_requested(cancel_flag: &Option<Arc<AtomicBool>>) -> bool {
    cancel_flag
        .as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
}

/// 写入并按限速分批等待
async fn write_with_limit(
    file: &mut std::fs::File,
    chunk: &[u8],
    rate_limiter: &Option<Arc<Mutex<RateLimiter>>>,
    downloaded: &mut u64,
    cancel_flag: &Option<Arc<AtomicBool>>,
) -> Result<(), BlockError> {
    let Some(limiter) = rate_limiter else {
        file.write_all(chunk)
            .map_err(|e| BlockError::Failed(format!("写入失败: {}", e)))?;
        *downloaded += chunk.len() as u64;
        return Ok(());
    };
    let mut remaining = chunk.len() as u64;
    let mut offset = 0_usize;
    let mut guard = limiter.lock().await;
    while remaining > 0 {
        if cancel_requested(cancel_flag) {
            return Err(BlockError::Cancelled);
        }
        let granted = guard.acquire(remaining);
        if granted == 0 {
            let wait = guard.wait_time_ms(remaining);
            drop(guard);
            tokio::time::sleep(Duration::from_millis(wait.max(10))).await;
            guard = limiter.lock().await;
            continue;
        }
        let end = (offset + granted as usize).min(chunk.len());
        file.write_all(&chunk[offset..end])
            .map_err(|e| BlockError::Failed(format!("写入失败: {}", e)))?;
        offset = end;
        remaining -= granted;
        *downloaded += granted;
    }
    Ok(())
}
