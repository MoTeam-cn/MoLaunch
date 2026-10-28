//! 分片提速：单源低速时把最难的分片拆成两半并行下载
//!
//! 只在「没有别的候选源」时启用：多源场景换源比拆分划算。拆分不改变分片布局，
//! 收尾时把「已下前缀 + 左半 + 右半」拼回原 `.part`，合并逻辑完全不变。

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

use crate::log_warn;
use crate::utils::format;

use super::super::rate_limiter::RateLimiter;
use super::super::types::GlobalProgress;
use super::download::download_chunk;

/// 拆分提速的最小剩余字节：太少拆开只多付一次往返开销
const MIN_SPLIT_BYTES: u64 = 8 * 1024 * 1024;

/// 分片任务句柄的返回类型
pub(super) type ChunkHandle =
    tokio::task::JoinHandle<(usize, Result<u64, Box<dyn std::error::Error + Send + Sync>>)>;

/// 拆分提速产生的两半文件
pub(super) struct SplitHalves {
    pub(super) index: usize,
    pub(super) left_path: String,
    pub(super) right_path: String,
}

/// 起一个分片任务
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_chunk_task(
    client: &reqwest::Client,
    url: &str,
    part_path: &str,
    index: usize,
    start: u64,
    end: u64,
    rate_limiter: &Arc<Mutex<RateLimiter>>,
    chunk_progress: &Arc<Vec<StdMutex<u64>>>,
    file_progress: &Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: &Option<Arc<AtomicBool>>,
    cancel_flag: &Option<Arc<AtomicBool>>,
    slow_flag: Option<Arc<AtomicBool>>,
) -> ChunkHandle {
    let client = client.clone();
    let url = url.to_string();
    let part_path = part_path.to_string();
    let limiter = rate_limiter.clone();
    let prog = chunk_progress.clone();
    let file_prog = file_progress.clone();
    let pause = pause_flag.clone();
    let cancel = cancel_flag.clone();
    tokio::spawn(async move {
        let result = download_chunk(
            &client, &url, &part_path, start, end, limiter, prog, index, file_prog, pause, cancel,
            slow_flag,
        )
        .await;
        (index, result)
    })
}

/// 计算拆分点：返回 (左半起点, 左半终点, 右半起点)
///
/// 左右两半都非空；区间只有 1 字节时无处可拆，返回 None。
fn plan_split_range(next_start: u64, end: u64) -> Option<(u64, u64, u64)> {
    if end <= next_start {
        return None;
    }
    let mid = next_start + (end - next_start) / 2;
    Some((next_start, mid, mid + 1))
}

/// 单源低速时把「最难的那个分片」的剩余区间一分为二并行下载
#[allow(clippy::too_many_arguments)]
pub(super) fn try_split_straggler(
    handles: &mut Vec<ChunkHandle>,
    local_path: &str,
    ranges: &[(u64, u64)],
    chunk_count: usize,
    client: &reqwest::Client,
    url: &str,
    rate_limiter: &Arc<Mutex<RateLimiter>>,
    chunk_progress: &Arc<Vec<StdMutex<u64>>>,
    pause_flag: &Option<Arc<AtomicBool>>,
    cancel_flag: &Option<Arc<AtomicBool>>,
) -> Option<SplitHalves> {
    // 找剩余字节最多、还在跑的那个分片
    let mut target: Option<(usize, u64, u64, u64)> = None;
    for (index, &(start, end)) in ranges.iter().enumerate() {
        if handles
            .get(index)
            .map(|handle| handle.is_finished())
            .unwrap_or(true)
        {
            continue;
        }
        let part_path = format!("{}.part{}", local_path, index);
        let already = std::fs::metadata(&part_path)
            .map(|meta| meta.len())
            .unwrap_or(0);
        let span = end.saturating_sub(start).saturating_add(1);
        if already >= span {
            continue;
        }
        let remaining = span - already;
        if target.is_none_or(|(_, _, _, best)| remaining > best) {
            target = Some((index, start + already, end, remaining));
        }
    }
    let (index, next_start, end, remaining) = target?;
    if remaining < MIN_SPLIT_BYTES {
        return None;
    }
    let (left_start, left_end, right_start) = plan_split_range(next_start, end)?;
    // 先中止原任务：它还会继续往后写，不中止会和拆分后的区间重叠
    if let Some(handle) = handles.get(index) {
        handle.abort();
    }
    let left_path = format!("{}.part{}.a", local_path, index);
    let right_path = format!("{}.part{}.b", local_path, index);
    log_warn!(
        "[Chunk] 单源低速，拆分分片 {} 的剩余 {} 并行提速: [{}, {}] + [{}, {}]",
        index,
        format::bytes(remaining),
        left_start,
        left_end,
        right_start,
        end
    );
    // 两半各自占用预留进度槽位；不带 file_progress（拼装成功后再一次性补账），
    // 也不带 slow 标志（否则会被已置位的慢速信号立刻打断）
    handles.push(spawn_chunk_task(
        client,
        url,
        &left_path,
        chunk_count,
        left_start,
        left_end,
        rate_limiter,
        chunk_progress,
        &None,
        pause_flag,
        cancel_flag,
        None,
    ));
    handles.push(spawn_chunk_task(
        client,
        url,
        &right_path,
        chunk_count + 1,
        right_start,
        end,
        rate_limiter,
        chunk_progress,
        &None,
        pause_flag,
        cancel_flag,
        None,
    ));
    Some(SplitHalves {
        index,
        left_path,
        right_path,
    })
}

/// 把「已下前缀 + 左半 + 右半」拼回原 `.part` 文件
pub(super) fn assemble_split(
    part_path: &str,
    left_path: &str,
    right_path: &str,
) -> std::io::Result<()> {
    let assembled = format!("{}.assembled", part_path);
    let mut output = std::fs::File::create(&assembled)?;
    for source in [part_path, left_path, right_path] {
        let mut input = std::fs::File::open(source)?;
        std::io::copy(&mut input, &mut output)?;
    }
    drop(output);
    std::fs::rename(&assembled, part_path)?;
    let _ = std::fs::remove_file(left_path);
    let _ = std::fs::remove_file(right_path);
    Ok(())
}

/// 追加已下载字节（拆分两半拼装成功后补账）
pub(super) fn add_progress(progress: &Arc<StdMutex<GlobalProgress>>, bytes: u64) {
    let mut guard = progress
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.downloaded_bytes = guard.downloaded_bytes.saturating_add(bytes);
}

#[cfg(test)]
#[path = "split_test.rs"]
mod split_test;
