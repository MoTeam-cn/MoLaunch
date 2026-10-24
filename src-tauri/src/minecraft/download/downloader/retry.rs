//! URL 顺序、重试与下载方式选择

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex;

use super::super::super::super::sources::DownloadSourceMode;
use super::super::super::rate_limiter::RateLimiter;
use super::super::super::types::{
    ContentValidator, DownloadProgress, DownloadStatus, DownloadTask, GlobalProgress,
};
use super::verify;
use crate::minecraft::download::chunk;
use crate::utils::format;
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
    content_validator: Option<ContentValidator>,
) -> Option<DownloadProgress> {
    // task.expected_size 是唯一可信的大小来源（清单/元数据给的）。
    // file_size 之后可能被「探测值」覆盖，而探测值只用于分片切分与进度估算，绝不可用于校验：
    // 部分服务端（如 Fabric 官方元数据）会把压缩后的长度写进 Content-Range（1401），
    // 实际落盘的是解压后的字节（2778），拿探测值校验必然误杀正常文件。
    let trusted_size = file_size;

    // 内容校验失败：删除文件、回滚已计进度、继续下一 URL（镜像坏文件自动回退官方等保底源）
    let invalid_content = |err: String, downloaded: u64| {
        log_warn!("文件内容校验失败：{} - {}", task.local_path, err);
        let _ = std::fs::remove_file(&task.local_path);
        rollback_progress(&progress, downloaded);
    };
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
                let probe = chunk::probe_range(client, url).await;
                if file_size == 0 && probe.total > 0 {
                    file_size = probe.total;
                }
                if probe.supports_range && chunkable_size(file_size, chunk_count) {
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
                        let expected = if trusted_size > 0 {
                            trusted_size
                        } else {
                            result.total
                        };
                        if let Some(err) = verify::check_chunk(task, expected) {
                            log_warn!("[Chunk] 文件校验失败：{} - {}", task.local_path, err);
                            let _ = std::fs::remove_file(&task.local_path);
                            // 回滚已计数进度：文件将被单流重新下载，避免 downloaded_bytes 重复累计
                            rollback_progress(&progress, result.downloaded);
                        } else {
                            // 大小校验通过后执行内容校验（如 zip 魔数），失败视为该源无效
                            if let Some(v) = &content_validator {
                                if let Err(err) = v(Path::new(&task.local_path)) {
                                    invalid_content(err, result.downloaded);
                                    continue 'url_loop;
                                }
                            }
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
                } else {
                    log_debug!(
                        "[Download] 不走分片（Range 支持: {}，大小: {}）: {}",
                        probe.supports_range,
                        format::bytes(file_size),
                        url
                    );
                }
            }
            log_debug!(
                "[Download] 从 {} 单流下载 (超时: {}s, 尝试 {}/{})",
                url,
                timeout.as_secs(),
                attempt,
                MAX_RETRIES
            );
            // 只传可信大小：探测值可能是压缩后长度，传进去会让字节上限与进度失真
            match super::super::stream::download_from_url(
                client,
                url,
                &task.local_path,
                trusted_size,
                rate_limiter.clone(),
                timeout,
                progress.clone(),
                pause_flag.clone(),
                cancel_flag.clone(),
            )
            .await
            {
                Ok((downloaded, total, speed)) => {
                    // 响应没给出可信长度（压缩响应 / chunked 传输）时不做大小校验：
                    // 此时的 expected_size 可能是压缩后长度或 HEAD 探测值，硬校验会误杀正常文件
                    let expected = stream_expected(trusted_size, total);
                    if let Some(err) = verify::check_stream(task, expected, downloaded) {
                        log_warn!("文件校验失败：{} - {}", task.local_path, err);
                        let _ = std::fs::remove_file(&task.local_path);
                        // 回滚已计数进度：文件将由下一个 URL 重新下载，避免进度重复累计
                        rollback_progress(&progress, downloaded);
                        continue 'url_loop;
                    }
                    // 大小校验通过后执行内容校验，失败视为该源无效
                    if let Some(v) = &content_validator {
                        if let Err(err) = v(Path::new(&task.local_path)) {
                            invalid_content(err, downloaded);
                            continue 'url_loop;
                        }
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

/// 单流校验用的期望大小
///
/// `response_total` 为响应自报长度（Content-Length）：为 0 说明服务端没给可信长度
/// （压缩响应被解码、或 chunked 传输），此时一律返回 0 交给 `check_stream` 按实际字节跳过校验。
fn stream_expected(trusted_size: u64, response_total: u64) -> u64 {
    if response_total > 0 {
        trusted_size
    } else {
        0
    }
}

/// 分片最小文件大小：低于此值分片只会多花往返开销，还会放大各源 Range 实现的差异
const MIN_CHUNK_BYTES: u64 = 1024 * 1024;

/// 是否值得分片下载
///
/// 只有「大小已知且不小于 1 MiB」才分片：大小未知时无法切分范围，
/// 小文件（如几 KB 的元数据 JSON）分片既无收益又极易踩到服务端 Range 的实现差异。
fn chunkable_size(file_size: u64, chunk_count: usize) -> bool {
    chunk_count > 1 && file_size >= MIN_CHUNK_BYTES
}

#[cfg(test)]
#[path = "retry_test.rs"]
mod retry_test;
