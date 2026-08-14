//! URL 顺序、重试与下载方式选择

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex;

use super::super::super::super::sources::DownloadSourceMode;
use super::super::super::rate_limiter::RateLimiter;
use super::super::super::types::{DownloadProgress, DownloadStatus, DownloadTask, GlobalProgress};
use super::verify;
use crate::minecraft::download::chunk;
use crate::{log_debug, log_warn};

const MAX_RETRIES: usize = 3;

#[allow(clippy::too_many_arguments)]
pub(super) async fn download_with_retries(
    client: &reqwest::Client,
    task: &DownloadTask,
    urls: &[String],
    chunk_count: usize,
    can_chunk: bool,
    mut file_size: u64,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    source_mode: DownloadSourceMode,
    progress: Option<Arc<StdMutex<GlobalProgress>>>,
    chunked_task_ids: Option<Arc<StdMutex<std::collections::HashSet<String>>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
) -> Option<DownloadProgress> {
    'url_loop: for url in urls {
        let timeout = request_timeout(source_mode, url);
        let mut chunk_disabled = false;
        for attempt in 1..=MAX_RETRIES {
            if is_cancelled(&cancel_flag) {
                break 'url_loop;
            }
            if can_chunk && !chunk_disabled {
                if attempt == 1 {
                    log_debug!("[Download] 检测分片支持: {}", url);
                }
                if chunk::supports_range(client, url).await {
                    let limiter = rate_limiter
                        .clone()
                        .unwrap_or_else(|| Arc::new(Mutex::new(RateLimiter::new(0))));
                    let result = chunk::download_chunked(
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
                    if file_size == 0 && result.total > 0 {
                        file_size = result.total;
                    }
                    if result.status == DownloadStatus::Completed {
                        if let Some(err) = verify::check_chunk(task, result.total) {
                            log_warn!("[Chunk] 文件校验失败：{} - {}", task.local_path, err);
                            let _ = std::fs::remove_file(&task.local_path);
                            // 回滚已计数进度：文件将被单流重新下载，避免 downloaded_bytes 重复累计
                            rollback_progress(&progress, result.downloaded);
                        } else {
                            if let Some(ref ids) = chunked_task_ids {
                                ids.lock().unwrap().insert(task.id.clone());
                            }
                            return Some(verify::completed(
                                task,
                                result.downloaded,
                                result.total,
                                result.speed,
                            ));
                        }
                    }
                    if result
                        .error
                        .as_ref()
                        .map(|err: &String| err.contains("404"))
                        .unwrap_or(false)
                    {
                        log_debug!("[Download] 分片返回 404，禁用分片改走单流: {}", url);
                        chunk_disabled = true;
                    }
                    log_debug!("[Chunk] 分片下载失败: {:?}, 回退单流", result.error);
                }
            }
            log_debug!(
                "[Download] 从 {} 单流下载 (超时: {}s, 尝试 {}/{})",
                url,
                timeout.as_secs(),
                attempt,
                MAX_RETRIES
            );
            match super::super::stream::download_from_url(
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
                    if let Some(err) = verify::check_stream(task, file_size, downloaded) {
                        log_warn!("文件校验失败：{} - {}", task.local_path, err);
                        let _ = std::fs::remove_file(&task.local_path);
                        // 回滚已计数进度：文件将由下一个 URL 重新下载，避免进度重复累计
                        rollback_progress(&progress, downloaded);
                        continue 'url_loop;
                    }
                    return Some(verify::completed(task, downloaded, total, speed));
                }
                Err(e) => {
                    log_debug!(
                        "从 {} 下载失败 (尝试 {}/{}): {}",
                        url,
                        attempt,
                        MAX_RETRIES,
                        e
                    );
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        }
    }
    None
}

fn request_timeout(source_mode: DownloadSourceMode, url: &str) -> Duration {
    match source_mode {
        DownloadSourceMode::Smart if url.contains("bmclapi") || url.contains("mirror") => {
            Duration::from_secs(10)
        }
        DownloadSourceMode::Smart => Duration::from_secs(5),
        _ => Duration::from_secs(30),
    }
}

fn is_cancelled(flag: &Option<Arc<AtomicBool>>) -> bool {
    flag.as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
}

/// 回滚已计入进度的字节（校验失败需重新下载时调用，避免 downloaded_bytes 重复累计虚高）
fn rollback_progress(progress: &Option<Arc<StdMutex<GlobalProgress>>>, bytes: u64) {
    if bytes == 0 {
        return;
    }
    if let Some(ref p) = progress {
        let mut p = p.lock().unwrap();
        p.downloaded_bytes = p.downloaded_bytes.saturating_sub(bytes);
    }
}
