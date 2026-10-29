//! 单文件下载编排入口

use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex;

use super::super::super::sources::DownloadSourceMode;
use super::super::super::utils::file_checker::FileChecker;
use super::super::job_store;
use super::super::rate_limiter::RateLimiter;
use super::super::types::{ContentValidator, DownloadProgress, DownloadTask, GlobalProgress};
use crate::log_debug;

/// 把 `.part` 落盘后原子替换到目标路径
///
/// 同目录 rename 是原子的：读者要么看到旧的完整文件，要么看到新的完整文件，
/// 不会看到写了一半的中间态。Windows 上 `fs::rename` 会替换已存在的目标文件。
///
/// 先 sync_all 再 rename：否则替换完成后内容还在页缓存里，掉电就成了「名字对、内容不完整」。
fn promote_part(part: &str, final_path: &str) -> Result<(), String> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(part)
        .map_err(|e| format!("打开临时文件失败 {}: {}", part, e))?;
    file.sync_all()
        .map_err(|e| format!("临时文件落盘失败 {}: {}", part, e))?;
    drop(file);
    std::fs::rename(part, final_path)
        .map_err(|e| format!("替换目标文件失败 {} -> {}: {}", part, final_path, e))
}

#[path = "engine.rs"]
mod engine;
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
    content_validator: Option<ContentValidator>,
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

    // 登记到台账：进程若中途被杀，下次启动还能看到这个任务没跑完
    let _job = job_store::JobGuard::new(&task.id, &task.local_path);
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

    // 先写 .part，校验通过再原子替换到目标路径：目标路径上永远不会出现
    // 「写到一半」的文件，断电或崩溃最多留下一个 .part 和一个位图
    let final_path = task.local_path.clone();
    let part_path = format!("{}.part", final_path);
    let working = DownloadTask {
        id: task.id.clone(),
        urls: task.urls.clone(),
        local_path: part_path.clone(),
        expected_size: task.expected_size,
        expected_hash: task.expected_hash.clone(),
    };

    let file_size = task.expected_size.max(0) as u64;
    // 分片门槛（大小下限、Range 支持）统一由 retry 内的 chunkable_size 判定
    let can_chunk = chunk_count > 1;
    let result = retry::download_with_retries(
        client,
        &working,
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
        content_validator,
    )
    .await;

    match result {
        retry::AttemptOutcome::Done(progress) => {
            // 校验都过了才替换：这一步之前目标路径上什么都没有，出问题也不会留半截文件
            if let Err(e) = promote_part(&part_path, &final_path) {
                return verify::failed(task, format!("落地替换失败：{}", e));
            }
            return progress;
        }
        // 半成品还在盘上，下一次同一任务会直接续；这里如实报告而不是「全部源失败」
        retry::AttemptOutcome::PartialKept => {
            log_debug!("[Download] 未完成但已保留半成品: {}", part_path);
            return verify::failed(task, "下载未完成，已保留半成品，下次将继续".to_string());
        }
        retry::AttemptOutcome::Failed => {}
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

#[cfg(test)]
#[path = "single_test.rs"]
mod single_test;
