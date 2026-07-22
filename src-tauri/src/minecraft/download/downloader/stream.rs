//! 单 URL 流式下载（HTTP 请求 + 限速 + 暂停/取消 + 进度更新）

use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use futures_util::StreamExt;

use super::super::rate_limiter::RateLimiter;
use super::super::types::GlobalProgress;

/// 从单个 URL 下载（支持限速和动态超时，实时更新进度）
pub(super) async fn download_from_url(
    client: &reqwest::Client,
    url: &str,
    local_path: &str,
    expected_size: u64,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    timeout: Duration,
    progress: Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> Result<(u64, u64, u64), Box<dyn std::error::Error + Send + Sync>> {
    let response = client.get(url).timeout(timeout).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP 错误：{}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let start_time = Instant::now();

    // 下载流字节数上限：已知期望大小时允许 2 倍冗余，否则使用绝对上限
    let byte_limit = if expected_size > 0 {
        expected_size.saturating_mul(2)
    } else {
        super::MAX_UNVERIFIED_BYTES
    };

    let mut stream = response.bytes_stream();
    // 确保父目录存在（取消/清理可能导致目录被删除）
    if let Some(parent) = Path::new(local_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = std::fs::File::create(local_path)?;

    // 回滚已增量加到 progress 的字节数（downloaded>0 时才需要）
    let rollback_progress = |downloaded: u64, progress: &Option<Arc<StdMutex<GlobalProgress>>>| {
        if downloaded > 0 {
            if let Some(ref p) = progress {
                let mut p = p.lock().unwrap();
                p.downloaded_bytes = p.downloaded_bytes.saturating_sub(downloaded);
            }
        }
    };

    while let Some(chunk) = stream.next().await {
        // 检查取消信号
        if let Some(ref flag) = cancel_flag {
            if flag.load(std::sync::atomic::Ordering::Relaxed) {
                rollback_progress(downloaded, &progress);
                return Err("下载已取消".into());
            }
        }
        // 检查暂停信号
        if let Some(ref flag) = pause_flag {
            while flag.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(ref cf) = cancel_flag {
                    if cf.load(std::sync::atomic::Ordering::Relaxed) {
                        rollback_progress(downloaded, &progress);
                        return Err("下载已取消".into());
                    }
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                rollback_progress(downloaded, &progress);
                return Err(Box::from(e) as Box<dyn std::error::Error + Send + Sync>);
            }
        };
        let chunk_size = chunk.len() as u64;

        // 限速处理
        if let Some(ref limiter) = rate_limiter {
            let mut limiter = limiter.lock().await;
            let mut remaining = chunk_size;
            let mut offset: usize = 0;

            while remaining > 0 {
                let granted = limiter.acquire(remaining);
                if granted == 0 {
                    let wait_ms = limiter.wait_time_ms(remaining);
                    drop(limiter);
                    tokio::time::sleep(Duration::from_millis(wait_ms.max(10))).await;
                    limiter = rate_limiter.as_ref().unwrap().lock().await;
                    continue;
                }

                let end = (offset + granted as usize).min(chunk.len());
                if let Err(e) = file.write_all(&chunk[offset..end]) {
                    rollback_progress(downloaded, &progress);
                    return Err(e.into());
                }
                offset = end;
                remaining -= granted;
                downloaded += granted;
            }
        } else {
            if let Err(e) = file.write_all(&chunk) {
                rollback_progress(downloaded, &progress);
                return Err(e.into());
            }
            downloaded += chunk_size;
        }

        // 增量更新全局进度（与分片下载保持一致，让前端实时看到下载进度）
        if let Some(ref p) = progress {
            let mut p = p.lock().unwrap();
            p.downloaded_bytes = p.downloaded_bytes.saturating_add(chunk_size);
        }

        // max_bytes 上限校验，防止被劫持镜像源返回无限流导致磁盘耗尽
        if downloaded > byte_limit {
            rollback_progress(downloaded, &progress);
            return Err(format!(
                "Download size exceeded limit: {} > {}",
                downloaded, byte_limit
            )
            .into());
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 {
        (downloaded as f64 / elapsed) as u64
    } else {
        0
    };

    Ok((downloaded, total_size, speed))
}
