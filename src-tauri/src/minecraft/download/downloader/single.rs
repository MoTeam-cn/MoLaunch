//! 单文件下载编排（URL 顺序循环 + 重试 + 分片/单流选择 + 校验）

use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use tokio::sync::Mutex;

use super::super::super::sources::DownloadSourceMode;
use super::super::super::utils::file_checker::FileChecker;
use super::super::rate_limiter::RateLimiter;
use super::super::types::{DownloadProgress, DownloadStatus, DownloadTask, GlobalProgress};
use crate::{log_debug, log_info, log_warn};

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
        log_debug!(
            "[Download] 跳过已存在文件: {} (size={})",
            task.local_path,
            task.expected_size
        );
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
        // 分片 404 时禁用分片：CF CDN 对部分文件的 Range 请求会返回 404，
        // 但完整 GET 请求正常。分片 404 后重试分片还会 404，直接走单流更高效。
        let mut chunk_disabled = false;

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
            // 取消信号检查：用户取消后立即跳出整个 URL 循环，不再继续尝试
            // （否则"下载已取消"错误会被当作普通失败重试 max_retries 次，浪费时间）
            if let Some(ref cf) = cancel_flag {
                if cf.load(std::sync::atomic::Ordering::Relaxed) {
                    log_info!("[Download] 检测到取消信号，停止重试: {}", task.local_path);
                    break 'url_loop;
                }
            }

            attempt += 1;

            // 尝试分片下载（大文件 + 服务器支持 Range + 未被 404 禁用）
            if can_chunk && !chunk_disabled {
                if attempt == 1 {
                    log_debug!("[Download] 检测分片支持: {}", url);
                }
                if super::super::chunk::supports_range(client, url).await {
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
                    let chunk_result = super::super::chunk::download_chunked(
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

                    // 分片返回 404 时禁用分片：CF CDN 对 Range 请求可能返回 404，
                    // 但完整 GET 正常。后续重试直接走单流，避免浪费分片探测时间。
                    if let Some(ref err) = chunk_result.error {
                        if err.contains("404") {
                            log_info!(
                                "[Download] 分片返回 404，禁用分片改走单流: {}",
                                url
                            );
                            chunk_disabled = true;
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
            match super::stream::download_from_url(
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
                        .with_actual_size(if file_size > 0 {
                            file_size as i64
                        } else {
                            downloaded as i64
                        })
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

    // 取消信号优先于失败：用户主动取消时返回"下载已取消"而非"所有下载源均失败"
    if let Some(ref cf) = cancel_flag {
        if cf.load(std::sync::atomic::Ordering::Relaxed) {
            log_info!("[Download] 下载已取消: {}", task.local_path);
            return DownloadProgress {
                task_id: task.id.clone(),
                downloaded: 0,
                total: 0,
                speed: 0,
                status: DownloadStatus::Failed,
                error: Some("下载已取消".to_string()),
            };
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
