//! 单文件下载编排入口

use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex;

use super::super::super::sources::DownloadSourceMode;
use super::super::super::utils::file_checker::FileChecker;
use super::super::rate_limiter::RateLimiter;
use super::super::types::{DownloadProgress, DownloadTask, GlobalProgress};
use crate::log_debug;

#[path = "retry.rs"]
mod retry;
#[path = "verify.rs"]
mod verify;

#[allow(clippy::too_many_arguments)]
/// 下载单个文件（顺序尝试 URL，自动在分片与单流之间选择）
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
    let checker = FileChecker::new()
        .with_actual_size(task.expected_size)
        .with_hash(task.expected_hash.clone());
    if checker.is_valid(&task.local_path) {
        log_debug!(
            "[Download] 跳过已存在文件: {} (size={})",
            task.local_path,
            task.expected_size
        );
        return verify::skipped(task);
    }

    log_debug!(
        "[Download] 开始下载: {} (size={}, urls={:?})",
        task.local_path,
        task.expected_size,
        urls
    );
    if let Some(parent) = Path::new(&task.local_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return verify::failed(task, format!("创建目录失败：{}", e));
        }
    }

    let file_size = task.expected_size.max(0) as u64;
    let can_chunk =
        chunk_count > 1 && (file_size == 0 || (file_size / chunk_count as u64) > 1_048_576);
    let result = retry::download_with_retries(
        client,
        task,
        urls,
        chunk_count,
        can_chunk,
        file_size,
        rate_limiter,
        source_mode,
        progress,
        chunked_task_ids,
        pause_flag,
        cancel_flag.clone(),
    )
    .await;

    if let Some(result) = result {
        return result;
    }
    if cancel_flag
        .as_ref()
        .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed))
    {
        log_debug!("[Download] 下载已取消: {}", task.local_path);
        return verify::failed(task, "下载已取消".to_string());
    }
    verify::failed(task, "所有下载源均失败".to_string())
}
