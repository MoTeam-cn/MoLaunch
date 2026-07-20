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

/// 未校验下载流的最大字节数上限，防止被劫持镜像源返回无限流导致磁盘耗尽
const MAX_UNVERIFIED_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB

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
    pause_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> DownloadProgress {
    // 检查文件是否已存在且有效
    let checker = FileChecker::new()
        .with_actual_size(task.expected_size)
        .with_hash(task.expected_hash.clone());

    if checker.is_valid(&task.local_path) {
        log_debug!("[Download] 跳过已存在文件: {} (size={})", task.local_path, task.expected_size);
        return DownloadProgress {
            task_id: task.id.clone(),
            downloaded: 0,
            total: task.expected_size.max(0) as u64,
            speed: 0,
            status: DownloadStatus::Skipped,
            error: None,
        };
    }

    log_debug!(
        "[Download] 开始下载: {} (size={}, urls={:?})",
        task.local_path,
        task.expected_size,
        urls
    );

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
    // file_size 可变：第一次分片探测到真实大小后记住，避免重试时重复探测
    // （重复探测会导致 progress.total_bytes 被 saturating_add 多次，前端显示总大小翻倍）
    let mut file_size = task.expected_size.max(0) as u64;

    // file_size 已知时按大小判断是否分片；file_size=0（未知大小）时直接尝试分片，
    // 由 chunk::download_chunked 内部探测真实大小并分片。
    // 这样整合包原始包（expected_size=0）能自动走分片，无需在调用方手动探测。
    let can_chunk = if file_size == 0 {
        chunk_count > 1
    } else {
        chunk_count > 1 && (file_size / chunk_count as u64) > chunk_threshold
    };

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
                        pause_flag.clone(),
                        cancel_flag.clone(),
                    )
                    .await;

                    // 第一次分片探测到真实大小后记住，避免重试时重复探测
                    // （重复探测会重复 saturating_add 到 progress.total_bytes，导致前端显示总大小翻倍）
                    if file_size == 0 && chunk_result.total > 0 {
                        file_size = chunk_result.total;
                    }

                    if chunk_result.status == DownloadStatus::Completed {
                        // 用 chunk_result.total（探测后的真实大小）校验
                        let checker = FileChecker::new()
                            .with_actual_size(chunk_result.total as i64)
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
                file_size,
                rate_limiter.clone(),
                timeout,
                progress.clone(),
                pause_flag.clone(),
                cancel_flag.clone(),
            )
            .await
            {
                Ok((downloaded, total, speed)) => {
                    // file_size=0（未探测，走单流回退）时用 downloaded 校验；
                    // file_size>0（已知大小）时用 file_size 校验
                    let checker = FileChecker::new()
                        .with_actual_size(if file_size > 0 { file_size as i64 } else { downloaded as i64 })
                        .with_hash(task.expected_hash.clone());

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

    log_warn!(
        "[Download] 下载失败: {} (尝试了 {} 个 URL)",
        task.local_path,
        urls.len()
    );
    DownloadProgress {
        task_id: task.id.clone(),
        downloaded: 0,
        total: 0,
        speed: 0,
        status: DownloadStatus::Failed,
        error: Some("所有下载源均失败".to_string()),
    }
}

/// 从单个 URL 下载（支持限速和动态超时，实时更新进度）
async fn download_from_url(
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
        MAX_UNVERIFIED_BYTES
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
