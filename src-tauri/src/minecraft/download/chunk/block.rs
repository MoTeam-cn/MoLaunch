//! 单块下载：Range 请求 + 随机写到目标文件的指定偏移

use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::Mutex;

use super::super::host_profile;
use super::super::mirror_gate;
use super::super::rate_limiter::RateLimiter;
use super::super::throttle;
use super::table::Segment;
use crate::minecraft::utils::hash::{HashAlgorithm, IncrementalHash};

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
    source_bytes: &AtomicU64,
    pause_flag: &Option<Arc<AtomicBool>>,
    cancel_flag: &Option<Arc<AtomicBool>>,
) -> Result<([u8; 32], u64), BlockError> {
    let expected = segment.len();
    // 这台主机先前已明确不支持 Range：这次请求注定白费，直接判负让调度换源
    if segment.start > 0 && host_profile::range_unsupported(url) {
        return Err(BlockError::RangeUnsupported);
    }

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
    if mirror_gate::observe(url, &response).is_some() {
        // 被限流了：镜像冷却之外再给监督循环一个「该退一档」的信号
        throttle::note_throttle();
    }
    let status = response.status();
    if status == reqwest::StatusCode::OK && segment.start > 0 {
        // 记下来：同一主机后面的块不必再白试一遍
        host_profile::note_range_unsupported(url);
        return Err(BlockError::RangeUnsupported);
    }
    if !status.is_success() {
        return Err(BlockError::Failed(format!("HTTP {}", status.as_u16())));
    }

    // 句柄进 Arc<Mutex>：写盘要挪到阻塞线程池里做，句柄得能跨线程持有
    let file = Arc::new(StdMutex::new(
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|e| BlockError::Failed(format!("打开目标文件失败: {}", e)))?,
    ));

    let mut downloaded = 0_u64;
    // 边写边算这块的 SHA256：位图靠它证明「这段字节确实落盘了」
    let mut hasher = IncrementalHash::new(HashAlgorithm::Sha256);
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
            &file,
            segment.start,
            &chunk,
            rate_limiter,
            &mut downloaded,
            cancel_flag,
        )
        .await?;
        hasher.update(&chunk);
        source_bytes.fetch_add(chunk_len, Ordering::Relaxed);
    }

    if downloaded != expected {
        return Err(BlockError::Failed(format!(
            "块大小不符：实际 {} 字节，期望 {}",
            downloaded, expected
        )));
    }
    let digest = hasher
        .finish_sha256()
        .map_err(|e| BlockError::Failed(format!("块哈希计算失败: {}", e)))?;
    Ok((digest, downloaded))
}

fn cancel_requested(cancel_flag: &Option<Arc<AtomicBool>>) -> bool {
    cancel_flag
        .as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
}

/// 把一段字节写到文件的指定偏移
///
/// 必须走 `spawn_blocking`：文件写入是同步系统调用，直接在 async 任务里写会把
/// 整个 reactor 一起卡住——一个文件几百块，每块都卡一下，并行度就白给了。
/// 用写句柄而不是只读句柄：只读句柄在 Windows 上连写都做不到。
async fn write_at(
    file: &Arc<StdMutex<std::fs::File>>,
    offset: u64,
    data: Vec<u8>,
) -> Result<(), BlockError> {
    let file = file.clone();
    tokio::task::spawn_blocking(move || {
        let mut guard = file.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        guard
            .seek(SeekFrom::Start(offset))
            .map_err(|e| BlockError::Failed(format!("定位写入偏移失败: {}", e)))?;
        guard
            .write_all(&data)
            .map_err(|e| BlockError::Failed(format!("写入失败: {}", e)))
    })
    .await
    .map_err(|e| BlockError::Failed(format!("写盘任务异常: {}", e)))?
}

async fn write_with_limit(
    file: &Arc<StdMutex<std::fs::File>>,
    base_offset: u64,
    chunk: &[u8],
    rate_limiter: &Option<Arc<Mutex<RateLimiter>>>,
    downloaded: &mut u64,
    cancel_flag: &Option<Arc<AtomicBool>>,
) -> Result<(), BlockError> {
    // 块内已写的位置：绝对偏移 = 块起点 + 已写字节数 + 块内偏移
    let block_start = base_offset + *downloaded;
    let Some(limiter) = rate_limiter else {
        write_at(file, block_start, chunk.to_vec()).await?;
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
        write_at(
            file,
            block_start + offset as u64,
            chunk[offset..end].to_vec(),
        )
        .await?;
        offset = end;
        remaining -= granted;
        *downloaded += granted;
    }
    Ok(())
}

#[cfg(test)]
#[path = "block_test.rs"]
mod block_test;
