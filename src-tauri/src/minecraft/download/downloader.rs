//! Single file download logic

use crate::{log_debug, log_info, log_warn};
use futures_util::StreamExt;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use super::super::sources::DownloadSourceMode;
use super::super::utils::file_checker::FileChecker;
use super::rate_limiter::RateLimiter;
use super::types::{DownloadProgress, DownloadStatus, DownloadTask, GlobalProgress};

/// 下载单个文件（统一逻辑：顺序尝试 URL，超时自动切换，大文件分片下载）
pub async fn download_single(
    client: &reqwest::Client,
    task: &DownloadTask,
    urls: &[String],
    chunk_count: usize,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    source_mode: DownloadSourceMode,
    progress: Option<Arc<StdMutex<GlobalProgress>>>,
    chunked_task_ids: Option<Arc<StdMutex<std::collections::HashSet<String>>>>,
) -> DownloadProgress {
    // 检查文件是否已存在且有效
    let checker = FileChecker::new()
        .with_actual_size(task.expected_size)
        .with_hash(task.expected_hash.clone());

    if checker.is_valid(&task.local_path) {
        return DownloadProgress {
            task_id: task.id.clone(),
            downloaded: 0,
            total: task.expected_size.max(0) as u64,
            speed: 0,
            status: DownloadStatus::Skipped,
            error: None,
        };
    }

    // 确保目录存在
    if let Some(parent) = Path::new(&task.local_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return DownloadProgress {
                task_id: task.id.clone(),
                downloaded: 0,
                total: 0,
                speed: 0,
                status: DownloadStatus::Failed,
                error: Some(format!("创建目录失败：{}", e)),
            };
        }
    }

    // 分片下载阈值：file_size / chunk_count > 1MB
    let chunk_threshold: u64 = 1_048_576;
    let file_size = task.expected_size.max(0) as u64;
    let can_chunk =
        chunk_count > 1 && file_size > 0 && (file_size / chunk_count as u64) > chunk_threshold;

    // 顺序尝试每个 URL，超时自动重试，总共最多 3 次
    let max_retries: usize = 3;

    'url_loop: for url in urls {
        // 每个 URL 独立计数，确保 URL 回退时重试次数正确重置
        let mut attempt = 0;

        // 确定超时时间
        let timeout = match source_mode {
            DownloadSourceMode::Smart => {
                if url.contains("bmclapi") || url.contains("mirror") {
                    Duration::from_secs(10)
                } else {
                    Duration::from_secs(5)
                }
            }
            _ => Duration::from_secs(30),
        };

        while attempt < max_retries {
            attempt += 1;

            // 尝试分片下载（大文件 + 服务器支持 Range）
            if can_chunk {
                if attempt == 1 {
                    log_debug!("[Download] 检测分片支持: {}", url);
                }
                if super::chunk::supports_range(client, url).await {
                    log_info!(
                        "[Download] 使用分片下载: {} ({} chunks, 尝试 {}/{})",
                        url,
                        chunk_count,
                        attempt,
                        max_retries
                    );
                    let limiter = rate_limiter
                        .clone()
                        .unwrap_or_else(|| Arc::new(Mutex::new(RateLimiter::new(0))));
                    let chunk_result = super::chunk::download_chunked(
                        client,
                        url,
                        &task.local_path,
                        file_size,
                        chunk_count,
                        limiter,
                        progress.clone(),
                    )
                    .await;

                    if chunk_result.status == DownloadStatus::Completed {
                        let checker = FileChecker::new()
                            .with_actual_size(task.expected_size)
                            .with_hash(task.expected_hash.clone());
                        if let Some(err) = checker.check(&task.local_path) {
                            log_warn!("[Chunk] 文件校验失败：{} - {}", task.local_path, err);
                            let _ = std::fs::remove_file(&task.local_path);
                        } else {
                            if let Some(ref ids) = chunked_task_ids {
                                ids.lock().unwrap().insert(task.id.clone());
                            }
                            return DownloadProgress {
                                task_id: task.id.clone(),
                                downloaded: chunk_result.downloaded,
                                total: chunk_result.total,
                                speed: chunk_result.speed,
                                status: DownloadStatus::Completed,
                                error: None,
                            };
                        }
                    }
                    log_debug!("[Chunk] 分片下载失败: {:?}, 回退单流", chunk_result.error);
                }
            }

            // 单流下载
            log_debug!(
                "[Download] 从 {} 单流下载 (超时: {}s, 尝试 {}/{})",
                url,
                timeout.as_secs(),
                attempt,
                max_retries
            );
            match download_from_url(
                client,
                url,
                &task.local_path,
                rate_limiter.clone(),
                timeout,
                progress.clone(),
            )
            .await
            {
                Ok((downloaded, total, speed)) => {
                    let checker = if task.expected_size == 0 && downloaded > 0 {
                        FileChecker::new()
                            .with_actual_size(downloaded as i64)
                            .with_hash(task.expected_hash.clone())
                    } else {
                        FileChecker::new()
                            .with_actual_size(task.expected_size)
                            .with_hash(task.expected_hash.clone())
                    };

                    if let Some(err) = checker.check(&task.local_path) {
                        log_warn!("文件校验失败：{} - {}", task.local_path, err);
                        let _ = std::fs::remove_file(&task.local_path);
                        continue 'url_loop;
                    }

                    return DownloadProgress {
                        task_id: task.id.clone(),
                        downloaded,
                        total,
                        speed,
                        status: DownloadStatus::Completed,
                        error: None,
                    };
                }
                Err(e) => {
                    log_debug!(
                        "从 {} 下载失败 (尝试 {}/{}): {}",
                        url,
                        attempt,
                        max_retries,
                        e
                    );
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        }
    }

    DownloadProgress {
        task_id: task.id.clone(),
        downloaded: 0,
        total: 0,
        speed: 0,
        status: DownloadStatus::Failed,
        error: Some("所有下载源均失败".to_string()),
    }
}

/// 从单个 URL 下载（支持限速和动态超时）
async fn download_from_url(
    client: &reqwest::Client,
    url: &str,
    local_path: &str,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    timeout: Duration,
    _progress: Option<Arc<StdMutex<GlobalProgress>>>,
) -> Result<(u64, u64, u64), Box<dyn std::error::Error + Send + Sync>> {
    let response = client.get(url).timeout(timeout).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP 错误：{}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let start_time = Instant::now();

    let mut stream = response.bytes_stream();
    let mut file = std::fs::File::create(local_path)?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
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
                file.write_all(&chunk[offset..end])?;
                offset = end;
                remaining -= granted;
                downloaded += granted;
            }
        } else {
            file.write_all(&chunk)?;
            downloaded += chunk_size;
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
