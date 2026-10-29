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
use crate::minecraft::download::credit::ProgressCredit;
use crate::minecraft::download::rate_limiter::RateLimiter;
use crate::minecraft::download::types::{
    ContentValidator, DownloadProgress, DownloadTask, GlobalProgress,
};
use crate::minecraft::sources::is_mirror_url;
use crate::{log_debug, log_warn};

/// 引擎一次尝试的结果
///
/// 关键区别在「半成品要不要留」：单流兜底会用 `File::create` 从头写，
/// 如果引擎刚把半成品与位图留在盘上，接着跑单流就等于把它们全丢了——
/// 所以必须把「留着别动」和「已清干净可以兜底」分成两种结果。
pub(super) enum EngineOutcome {
    /// 成功
    Done(DownloadProgress),
    /// 有半成品与位图保留在盘上，不能再走单流（会覆盖掉）
    KeptPartial,
    /// 彻底失败且已清理，可以交给单流兜底
    Fallback,
}

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
) -> EngineOutcome {
    let key = resume_key(task, file_size);
    // 进度记账：每轮跑完把「本文件当前可信字节」写进去，内部按差额增减，
    // 作废块重下时自动退回虚高的部分
    let credit = ProgressCredit::new(progress);
    // 「主源」= 第一条非镜像 URL：隔离块的重下与校验失败后的定向重下都应该找它。
    // 直接写死 0 会踩到「用户自配镜像排在第一位」的情况，那时 0 号其实是镜像。
    let primary = urls.iter().position(|url| !is_mirror_url(url)).unwrap_or(0);
    let mut outcome = engine_once(
        client,
        urls,
        RetryScope::Full,
        key,
        primary,
        task,
        file_size,
        chunk_count,
        rate_limiter,
        Some(&credit),
        pause_flag,
        cancel_flag,
    )
    .await;
    credit.credit(outcome.downloaded);
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
            primary,
            task,
            file_size,
            chunk_count,
            rate_limiter,
            Some(&credit),
            pause_flag,
            cancel_flag,
        )
        .await;
        credit.credit(outcome.downloaded);
    }
    let mut usable = verify_result(task, &outcome, trusted_size, content_validator);
    if !usable && outcome.complete && !is_cancelled(cancel_flag) {
        outcome = engine_once(
            client,
            urls,
            RetryScope::Suspects(primary),
            key,
            primary,
            task,
            file_size,
            chunk_count,
            rate_limiter,
            Some(&credit),
            pause_flag,
            cancel_flag,
        )
        .await;
        credit.credit(outcome.downloaded);
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
            RetryScope::Everything(primary),
            key,
            primary,
            task,
            file_size,
            chunk_count,
            rate_limiter,
            Some(&credit),
            pause_flag,
            cancel_flag,
        )
        .await;
        credit.credit(outcome.downloaded);
        usable = verify_result(task, &outcome, trusted_size, content_validator);
    }
    if usable {
        if let Some(ref ids) = chunked_task_ids {
            ids.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(task.id.clone());
        }
        return EngineOutcome::Done(verify::completed(
            task,
            outcome.downloaded,
            outcome.total,
            0,
        ));
    }
    // 有进度但没跑完（断网、反复超时、用户取消）：保留半成品与位图，
    // 下次同一任务进来直接续——取消不该把已经下好的几十兆一起扔掉
    if outcome.resumable {
        log_warn!(
            "[Download] 保留半成品与位图（{}/{} 字节），下次继续: {}",
            outcome.downloaded,
            outcome.total,
            task.local_path
        );
        credit.rollback();
        return EngineOutcome::KeptPartial;
    }
    // 一点进度都没有，或数据已不可信：清干净交给单流兜底
    log_debug!(
        "[Download] 并行下载未成功，改用单流兜底: {:?}",
        outcome.error
    );
    let _ = std::fs::remove_file(&task.local_path);
    let _ = std::fs::remove_file(format!("{}.partmap", task.local_path));
    credit.rollback();
    EngineOutcome::Fallback
}

/// 跑一轮段表引擎
#[allow(clippy::too_many_arguments)]
async fn engine_once(
    client: &reqwest::Client,
    urls: &[String],
    scope: RetryScope,
    resume_key: Option<u64>,
    official_index: usize,
    task: &DownloadTask,
    file_size: u64,
    chunk_count: usize,
    rate_limiter: &Option<Arc<Mutex<RateLimiter>>>,
    credit: Option<&ProgressCredit<'_>>,
    pause_flag: &Option<Arc<AtomicBool>>,
    cancel_flag: &Option<Arc<AtomicBool>>,
) -> chunk::SegmentOutcome {
    chunk::download_multi(
        client,
        urls,
        official_index,
        scope,
        resume_key,
        &task.local_path,
        file_size,
        chunk_count.max(1),
        rate_limiter.clone(),
        credit,
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

fn is_cancelled(flag: &Option<Arc<AtomicBool>>) -> bool {
    flag.as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
}

/// 续传指纹：同一任务、同一期望内容才算「同一次下载」
///
/// 位图靠它拒绝复用过期的半成品：同名文件大小相同但内容换了的场景（同名模组更新），
/// 只比较文件大小是分辨不出来的，所以期望哈希必须参与。
///
/// 候选源列表**不**参与：它只说明「从哪儿取」，不改变期望内容。用户切换下载源模式时
/// 列表会被重排，把源算进指纹的话，切一次就把已下好的块全部作废了。
///
/// 没有期望哈希时返回 `None`：拼接出来的内容无从裁决，此时不允许续传。
fn resume_key(task: &DownloadTask, file_size: u64) -> Option<u64> {
    let expected = task.expected_hash.as_deref().unwrap_or("");
    if expected.is_empty() {
        return None;
    }
    let mut hash = FNV_OFFSET;
    feed(&mut hash, task.id.as_bytes());
    feed(&mut hash, expected.as_bytes());
    feed(&mut hash, &file_size.to_le_bytes());
    Some(hash)
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
