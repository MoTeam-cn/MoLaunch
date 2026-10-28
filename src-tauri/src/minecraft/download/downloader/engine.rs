//! 段表引擎的调用阶梯：跑一轮 → 校验 → 按可疑范围重跑
//!
//! 元数据只给整份哈希、不给块级哈希，所以「哪一块坏了」只能靠来源缩小范围：
//! 全部 → 只重下非官方源的块 → 全部作废只用官方源。

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

use super::verify;
use crate::minecraft::download::chunk;
use crate::minecraft::download::chunk::RetryScope;
use crate::minecraft::download::rate_limiter::RateLimiter;
use crate::minecraft::download::types::{
    ContentValidator, DownloadProgress, DownloadTask, GlobalProgress,
};
use crate::{log_debug, log_warn};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// 跑段表引擎直到通过校验，失败时按规则清理或保留半成品
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_engine(
    client: &reqwest::Client,
    urls: &[String],
    task: &DownloadTask,
    file_size: u64,
    chunk_count: usize,
    trusted_size: u64,
    rate_limiter: &Option<Arc<Mutex<RateLimiter>>>,
    progress: &Option<Arc<StdMutex<GlobalProgress>>>,
    chunked_task_ids: &Option<Arc<StdMutex<std::collections::HashSet<String>>>>,
    pause_flag: &Option<Arc<AtomicBool>>,
    cancel_flag: &Option<Arc<AtomicBool>>,
    content_validator: &Option<ContentValidator>,
) -> Option<DownloadProgress> {
    let key = resume_key(task, urls, file_size);
    // 本文件已计入全局进度的字节：每轮跑完按「当前可信字节」对齐，
    // 作废块重下时把虚高的部分退回去，保证进度与实际落盘一致
    let mut counted = 0_u64;
    let mut outcome = engine_once(
        client,
        urls,
        RetryScope::Full,
        key,
        task,
        file_size,
        chunk_count,
        rate_limiter,
        progress,
        pause_flag,
        cancel_flag,
    )
    .await;
    sync_progress(progress, &mut counted, outcome.downloaded);
    // 首轮未完成：位图里已有完成块，再跑一轮只补没完成的
    if !outcome.complete && !is_cancelled(cancel_flag) {
        log_warn!(
            "[Download] 并行下载首轮未完成，复用已完成块重试: {:?}",
            outcome.error
        );
        outcome = engine_once(
            client,
            urls,
            RetryScope::Full,
            key,
            task,
            file_size,
            chunk_count,
            rate_limiter,
            progress,
            pause_flag,
            cancel_flag,
        )
        .await;
        sync_progress(progress, &mut counted, outcome.downloaded);
    }
    let mut usable = verify_result(task, &outcome, trusted_size, content_validator);
    if !usable && outcome.complete && !is_cancelled(cancel_flag) {
        outcome = engine_once(
            client,
            urls,
            RetryScope::Suspects(0),
            key,
            task,
            file_size,
            chunk_count,
            rate_limiter,
            progress,
            pause_flag,
            cancel_flag,
        )
        .await;
        sync_progress(progress, &mut counted, outcome.downloaded);
        usable = verify_result(task, &outcome, trusted_size, content_validator);
    }
    if !usable && outcome.complete && !is_cancelled(cancel_flag) {
        log_warn!(
            "[Download] 排除来源后仍校验失败，改为全部重下: {}",
            task.local_path
        );
        outcome = engine_once(
            client,
            urls,
            RetryScope::Everything(0),
            key,
            task,
            file_size,
            chunk_count,
            rate_limiter,
            progress,
            pause_flag,
            cancel_flag,
        )
        .await;
        sync_progress(progress, &mut counted, outcome.downloaded);
        usable = verify_result(task, &outcome, trusted_size, content_validator);
    }
    if usable {
        if let Some(ref ids) = chunked_task_ids {
            ids.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(task.id.clone());
        }
        return Some(verify::completed(
            task,
            outcome.downloaded,
            outcome.total,
            0,
        ));
    }
    // 有进度但没跑完（断网/反复超时）：保留半成品与位图，下次同一任务进来直接续
    if outcome.resumable && !is_cancelled(cancel_flag) {
        log_warn!(
            "[Download] 保留半成品与位图（{}/{} 字节），下次继续: {}",
            outcome.downloaded,
            outcome.total,
            task.local_path
        );
        rollback_progress(progress, counted);
        return None;
    }
    // 一点进度都没有，或数据已不可信：清干净交给单流兜底
    log_debug!(
        "[Download] 并行下载未成功，改用单流兜底: {:?}",
        outcome.error
    );
    let _ = std::fs::remove_file(&task.local_path);
    let _ = std::fs::remove_file(format!("{}.partmap", task.local_path));
    rollback_progress(progress, counted);
    None
}

/// 跑一轮段表引擎
#[allow(clippy::too_many_arguments)]
async fn engine_once(
    client: &reqwest::Client,
    urls: &[String],
    scope: RetryScope,
    resume_key: u64,
    task: &DownloadTask,
    file_size: u64,
    chunk_count: usize,
    rate_limiter: &Option<Arc<Mutex<RateLimiter>>>,
    progress: &Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: &Option<Arc<AtomicBool>>,
    cancel_flag: &Option<Arc<AtomicBool>>,
) -> chunk::SegmentOutcome {
    chunk::download_multi(
        client,
        urls,
        0,
        scope,
        resume_key,
        &task.local_path,
        file_size,
        chunk_count.max(1),
        rate_limiter.clone(),
        progress.clone(),
        pause_flag.clone(),
        cancel_flag.clone(),
    )
    .await
}

/// 引擎结果的最终校验：大小 + 内容
fn verify_result(
    task: &DownloadTask,
    outcome: &chunk::SegmentOutcome,
    trusted_size: u64,
    content_validator: &Option<ContentValidator>,
) -> bool {
    if !outcome.complete {
        return false;
    }
    let expected = if trusted_size > 0 {
        trusted_size
    } else {
        outcome.total
    };
    if let Some(err) = verify::check_chunk(task, expected) {
        log_warn!(
            "[Download] 大小/哈希校验失败：{} - {}",
            task.local_path,
            err
        );
        return false;
    }
    if let Some(validator) = content_validator {
        if let Err(err) = validator(Path::new(&task.local_path)) {
            log_warn!("[Download] 内容校验失败: {}", err);
            return false;
        }
    }
    true
}

/// 让全局进度与本文件「当前可信字节」对齐
fn sync_progress(
    progress: &Option<Arc<StdMutex<GlobalProgress>>>,
    counted: &mut u64,
    current: u64,
) {
    if current > *counted {
        add_progress(progress, current - *counted);
    } else if current < *counted {
        rollback_progress(progress, *counted - current);
    }
    *counted = current;
}

fn add_progress(progress: &Option<Arc<StdMutex<GlobalProgress>>>, bytes: u64) {
    if bytes == 0 {
        return;
    }
    if let Some(ref p) = progress {
        let mut p = p.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        p.downloaded_bytes = p.downloaded_bytes.saturating_add(bytes);
    }
}

fn rollback_progress(progress: &Option<Arc<StdMutex<GlobalProgress>>>, bytes: u64) {
    if bytes == 0 {
        return;
    }
    if let Some(ref p) = progress {
        let mut p = p.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        p.downloaded_bytes = p.downloaded_bytes.saturating_sub(bytes);
    }
}

fn is_cancelled(flag: &Option<Arc<AtomicBool>>) -> bool {
    flag.as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
}

/// 续传指纹：同一任务、同一来源集合、同一期望内容才算「同一次下载」
///
/// 位图靠它拒绝复用过期的半成品：同名文件大小相同但内容换了的场景（同名模组更新），
/// 只比较文件大小是分辨不出来的。
fn resume_key(task: &DownloadTask, urls: &[String], file_size: u64) -> u64 {
    let mut hash = FNV_OFFSET;
    feed(&mut hash, task.id.as_bytes());
    feed(
        &mut hash,
        task.expected_hash.as_deref().unwrap_or("").as_bytes(),
    );
    feed(&mut hash, &file_size.to_le_bytes());
    for url in urls {
        feed(&mut hash, &[0_u8]);
        feed(&mut hash, url.as_bytes());
    }
    hash
}

fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod engine_test;
