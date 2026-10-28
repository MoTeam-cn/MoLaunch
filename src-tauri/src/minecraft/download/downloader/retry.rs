//! 单文件下载：段表引擎优先，小文件与探测不到大小的走单流兜底

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex;

use super::super::super::super::sources::DownloadSourceMode;
use super::super::super::edge;
use super::super::super::mirror_gate;
use super::super::super::rate_limiter::RateLimiter;
use super::super::super::slow;
use super::super::super::types::{
    ContentValidator, DownloadProgress, DownloadTask, GlobalProgress,
};
use super::engine::{self, EngineOutcome};
use super::verify;
use crate::minecraft::download::chunk;
use crate::{log_debug, log_warn};

const MAX_RETRIES: usize = 3;

/// 一次下载尝试的结果
///
/// 与单纯的 `Option` 相比多了一个「半成品已保留」：此时既不能算成功，
/// 也不能让上层再拿别的路径去覆盖那个文件。
pub(super) enum AttemptOutcome {
    /// 成功
    Done(DownloadProgress),
    /// 未完成，但半成品与位图已保留在盘上
    PartialKept,
    /// 所有路径都失败
    Failed,
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn download_with_retries(
    client: &reqwest::Client,
    task: &DownloadTask,
    urls: &[String],
    chunk_count: usize,
    can_chunk: bool,
    file_size: u64,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    source_mode: DownloadSourceMode,
    progress: Option<Arc<StdMutex<GlobalProgress>>>,
    chunked_task_ids: Option<Arc<StdMutex<std::collections::HashSet<String>>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
    content_validator: Option<ContentValidator>,
) -> AttemptOutcome {
    if urls.is_empty() {
        return AttemptOutcome::Failed;
    }
    let trusted_size = file_size;

    // 段表引擎要按块切分，大小未知时先探一次；探不到就交给单流
    let mut size = file_size;
    if size == 0 {
        size = chunk::probe_file_size(client, urls[0].as_str()).await;
    }

    if can_chunk && chunkable_size(size, chunk_count) && !is_cancelled(&cancel_flag) {
        match engine::run_engine(
            client,
            urls,
            task,
            size,
            chunk_count,
            trusted_size,
            &rate_limiter,
            &progress,
            &chunked_task_ids,
            &pause_flag,
            &cancel_flag,
            &content_validator,
        )
        .await
        {
            EngineOutcome::Done(progress) => return AttemptOutcome::Done(progress),
            // 半成品与位图留着，别让单流把它覆盖掉：
            // 下一次同一任务进来会直接续，比重新下一遍划算
            EngineOutcome::KeptPartial => return AttemptOutcome::PartialKept,
            EngineOutcome::Fallback => {}
        }
    }

    match download_single_stream(
        client,
        task,
        urls,
        trusted_size,
        rate_limiter,
        source_mode,
        progress,
        pause_flag,
        cancel_flag,
        content_validator,
    )
    .await
    {
        Some(progress) => AttemptOutcome::Done(progress),
        None => AttemptOutcome::Failed,
    }
}

/// 单流兜底：小文件、探测不到大小，或引擎彻底失败时使用
#[allow(clippy::too_many_arguments)]
async fn download_single_stream(
    client: &reqwest::Client,
    task: &DownloadTask,
    urls: &[String],
    trusted_size: u64,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    source_mode: DownloadSourceMode,
    progress: Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
    content_validator: Option<ContentValidator>,
) -> Option<DownloadProgress> {
    for (url_index, url) in urls.iter().enumerate() {
        if is_cancelled(&cancel_flag) {
            return None;
        }
        if let Some(remaining) = mirror_gate::cooldown_remaining(url) {
            if url_index + 1 < urls.len() {
                log_warn!(
                    "[Download] 镜像冷却中（剩余 {:.1}s），换源: {}",
                    remaining.as_secs_f64(),
                    url
                );
                continue;
            }
            let wait = remaining.min(mirror_gate::MAX_COOLDOWN_WAIT);
            log_debug!(
                "[Download] 镜像冷却中（剩余 {:.1}s），等待后重试: {}",
                remaining.as_secs_f64(),
                url
            );
            tokio::time::sleep(wait).await;
        }
        let timeout = request_timeout(source_mode, url.as_str());
        let has_next_url = url_index + 1 < urls.len();
        // 边缘优选：命中缓存时换用「钉住该 IP」的客户端，证书与 Host 仍是原域名
        let url_client = edge::client_for(url.as_str(), client);
        for attempt in 1..=MAX_RETRIES {
            if is_cancelled(&cancel_flag) {
                return None;
            }
            log_debug!(
                "[Download] 从 {} 单流下载 (超时: {}s, 尝试 {}/{})",
                url,
                timeout.as_secs(),
                attempt,
                MAX_RETRIES
            );
            match super::super::stream::download_from_url(
                &url_client,
                url.as_str(),
                &task.local_path,
                trusted_size,
                has_next_url,
                rate_limiter.clone(),
                timeout,
                progress.clone(),
                pause_flag.clone(),
                cancel_flag.clone(),
            )
            .await
            {
                Ok((downloaded, total, speed)) => {
                    // 响应没给出可信长度时不做大小校验（可能是压缩后长度或探测值）
                    let expected = stream_expected(trusted_size, total);
                    if let Some(err) = verify::check_stream(task, expected, downloaded) {
                        log_warn!("文件校验失败：{} - {}", task.local_path, err);
                        let _ = std::fs::remove_file(&task.local_path);
                        rollback_progress(&progress, downloaded);
                        break;
                    }
                    if let Some(validator) = &content_validator {
                        if let Err(err) = validator(Path::new(&task.local_path)) {
                            log_warn!("文件内容校验失败：{} - {}", task.local_path, err);
                            let _ = std::fs::remove_file(&task.local_path);
                            rollback_progress(&progress, downloaded);
                            break;
                        }
                    }
                    // 单流接手后位图已失效，顺手清掉避免下次误复用
                    let _ = std::fs::remove_file(format!("{}.partmap", task.local_path));
                    return Some(verify::completed(task, downloaded, total, speed));
                }
                Err(e) => {
                    // 线路判慢：还有别的候选就立刻换源，别在同一个 URL 上耗三次
                    if has_next_url && e.downcast_ref::<slow::SlowRouteError>().is_some() {
                        log_warn!("[Download] 线路持续低速，换源: {}", url);
                        break;
                    }
                    // 传输层失败计入边缘节点健康度：连续两次就作废该节点重新优选
                    edge::note_failure(url.as_str());
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

/// 回滚已计入进度的字节（校验失败需重新下载时调用，避免进度虚高）
fn rollback_progress(progress: &Option<Arc<StdMutex<GlobalProgress>>>, bytes: u64) {
    if bytes == 0 {
        return;
    }
    if let Some(ref p) = progress {
        let mut p = p.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        p.downloaded_bytes = p.downloaded_bytes.saturating_sub(bytes);
    }
}

/// 单流校验用的期望大小
///
/// `response_total` 为响应自报长度：为 0 说明服务端没给可信长度（压缩响应被解码、
/// 或 chunked 传输），此时返回 0 交给 `check_stream` 按实际字节跳过校验。
fn stream_expected(trusted_size: u64, response_total: u64) -> u64 {
    if response_total > 0 {
        trusted_size
    } else {
        0
    }
}

/// 分片最小文件大小：低于此值分片只会多花往返开销，还会放大各源 Range 实现的差异
const MIN_CHUNK_BYTES: u64 = 1024 * 1024;

/// 是否值得走段表引擎
///
/// 只有「大小已知且不小于 1 MiB」才值得按块切分：大小未知时无法切块，
/// 小文件（几 KB 的元数据 JSON）分块既无收益又容易踩到服务端 Range 的实现差异。
fn chunkable_size(file_size: u64, chunk_count: usize) -> bool {
    chunk_count > 1 && file_size >= MIN_CHUNK_BYTES
}

#[cfg(test)]
#[path = "retry_test.rs"]
mod retry_test;
