//! 分片下载调度：ChunkDownloadResult / download_chunked

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;
use tokio::sync::Mutex;

use crate::utils::format;
use crate::{log_debug, log_warn};

use super::super::rate_limiter::RateLimiter;
use super::super::slow;
use super::super::types::{DownloadStatus, GlobalProgress};
use super::download::download_chunk;
use super::merge::merge_chunks;
use super::probe::probe_file_size;
use super::split::{
    add_progress, assemble_split, spawn_chunk_task, try_split_straggler, ChunkHandle, SplitHalves,
};

/// 目标分片字节数：文件越大分片越多
///
/// 分段过小只会放大各源 Range 实现的差异与往返开销；16 MiB 是「并行收益明显、
/// 单片失败代价可接受」的折中值。
const TARGET_SEGMENT_BYTES: u64 = 16 * 1024 * 1024;

/// 分片数硬上限（在用户设置之外最多再翻一倍，且不超过这个值）
const MAX_PLANNED_CHUNKS: usize = 12;

/// 拆分提速需要的额外进度槽位（左右两半各一个）
const SPLIT_SLOTS: usize = 2;

/// 拆分提速的监督轮询间隔
const SPLIT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

/// 清理某个文件的所有分片临时文件
///
/// 分片布局一旦改变（换源升档并行度），旧 `.part` 的字节偏移就不再对应新的
/// Range 区间，续传会把错误偏移当成已下载量，合并出来是坏文件。所以改变布局
/// 之前必须清干净。
pub fn clear_part_files(local_path: &str) {
    for index in 0..=MAX_PLANNED_CHUNKS {
        let _ = std::fs::remove_file(format!("{}.part{}", local_path, index));
    }
}

/// 规划实际使用的分片数
///
/// 用户设置的分片数是**下限**：大文件按 [`TARGET_SEGMENT_BYTES`] 推导出更多分片，
/// 最多翻一倍且不超过 [`MAX_PLANNED_CHUNKS`]；小文件保持用户设置，不做缩减。
///
/// 规划只依赖 (文件大小, 用户设置, 档位)：同一档位下多次重试或换源都会得到相同的
/// 分片布局，`.part` 分片才能被续传复用；换源升档会改变布局，调用方需先清分片。
///
/// `boost` 是换源升档系数：首次尝试为 1，判慢或失败换源后翻倍，让「第一次没跑满
/// 带宽」的大文件在下一条线路上拿到更高的并行度。
fn plan_chunk_count(file_size: u64, requested: usize, boost: u32) -> usize {
    if requested <= 1 {
        return requested;
    }
    let target = requested.saturating_mul(boost.max(1) as usize);
    // 上限：最多再翻一倍，且不超过硬上限（clamp 的 min 恒小于 max，不会 panic）
    let upper = target.saturating_mul(2).clamp(1, MAX_PLANNED_CHUNKS);
    let lower = target.min(upper);
    let by_size = usize::try_from(file_size / TARGET_SEGMENT_BYTES).unwrap_or(upper);
    by_size.clamp(lower, upper)
}

/// 采样任务的 RAII 守卫：本函数任意路径返回都会中止采样，避免任务泄漏
struct SlowSamplerGuard(Option<tokio::task::JoinHandle<()>>);

impl Drop for SlowSamplerGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.0.as_ref() {
            handle.abort();
        }
    }
}

/// 分片下载结果
pub struct ChunkDownloadResult {
    pub downloaded: u64,
    pub total: u64,
    pub speed: u64,
    pub status: DownloadStatus,
    pub error: Option<String>,
    /// 是否因「线路持续低速」被中止：调用方应换源，而不是在本 URL 上继续重试
    pub slow: bool,
}

#[allow(clippy::too_many_arguments)]
/// 分片下载单个文件
///
/// - `file_size` 为 0 时会自动探测（GET + Range:bytes=0-0，通过 Content-Range 拿总大小）
/// - `chunk_count` 分片数量
/// - 所有 chunk 共享同一个 RateLimiter
/// - 进度通过 `file_progress` 实时更新，供速度计算使用
pub async fn download_chunked(
    client: &reqwest::Client,
    url: &str,
    local_path: &str,
    mut file_size: u64,
    chunk_count: usize,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    file_progress: Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
    allow_slow_abort: bool,
    segment_boost: u32,
) -> ChunkDownloadResult {
    // file_size=0 时自动探测（GET + Range:bytes=0-0，通过 Content-Range 拿总大小）
    // 不用 HEAD 是因为 Modrinth CDN 307 重定向后 HEAD 不返回 Content-Length
    if file_size == 0 {
        file_size = probe_file_size(client, url).await;
        if file_size == 0 {
            return ChunkDownloadResult {
                downloaded: 0,
                total: 0,
                speed: 0,
                status: DownloadStatus::Failed,
                error: Some("无法探测文件大小".into()),
                slow: false,
            };
        }
        // 探测到的真实大小回写到全局 total_bytes（download_batch 初始化时按 expected_size=0 求和，
        // total_bytes 为 0，前端会一直显示「计算中...」）
        if let Some(ref fp) = file_progress {
            let mut p = fp.lock().unwrap();
            p.total_bytes = p.total_bytes.saturating_add(file_size);
        }
    }

    // 大文件按大小放大分片数（用户设置为下限），小文件保持原设置
    let chunk_count = plan_chunk_count(file_size, chunk_count, segment_boost);

    if file_size == 0 || chunk_count <= 1 {
        return ChunkDownloadResult {
            downloaded: 0,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some("invalid chunk params".into()),
            slow: false,
        };
    }

    if let Some(parent) = Path::new(local_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ChunkDownloadResult {
                downloaded: 0,
                total: file_size,
                speed: 0,
                status: DownloadStatus::Failed,
                error: Some(format!("创建目录失败：{}", e)),
                slow: false,
            };
        }
    }

    let chunk_size = file_size / chunk_count as u64;
    if chunk_size == 0 {
        // file_size < chunk_count，无法分片，避免整数下溢
        return ChunkDownloadResult {
            downloaded: 0,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some("file size too small to chunk".into()),
            slow: false,
        };
    }
    let mut ranges: Vec<(u64, u64)> = Vec::with_capacity(chunk_count);

    for i in 0..chunk_count {
        let start = i as u64 * chunk_size;
        let end = if i == chunk_count - 1 {
            file_size - 1
        } else {
            start + chunk_size - 1
        };
        ranges.push((start, end));
    }

    log_debug!(
        "[Chunk] 开始分片下载: {} ({}, {} chunks, 每块约 {})",
        local_path,
        format::bytes(file_size),
        chunk_count,
        format::bytes(chunk_size)
    );

    // 分片进度追踪：每个 chunk 已下载字节数；末尾两个槽位留给「拆分提速」的两半
    // （采样器按全部槽位求和，所以拆出来的两半会自然计入整文件吞吐）
    let chunk_progress: Arc<Vec<StdMutex<u64>>> = Arc::new(
        (0..chunk_count + SPLIT_SLOTS)
            .map(|_| StdMutex::new(0u64))
            .collect(),
    );

    // 慢速采样：每 0.5s 汇总全文件已下字节，按 5s 窗口判定聚合吞吐。
    // 只看单条连接会把「8 条各 100KiB/s」误判成低速，所以必须按整文件聚合。
    let slow_abort = Arc::new(AtomicBool::new(false));
    let sampler_slow_abort = slow_abort.clone();
    let sampler_progress = chunk_progress.clone();
    // 只有「后面还有别的候选源」时才需要判慢（见 retry），否则连采样任务都不必起
    let _slow_sampler = SlowSamplerGuard(if allow_slow_abort {
        Some(tokio::spawn(async move {
            let mut window_started = Instant::now();
            let mut window_bytes = 0_u64;
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if sampler_slow_abort.load(Ordering::Relaxed) {
                    return;
                }
                let elapsed = window_started.elapsed();
                if elapsed < slow::WINDOW {
                    continue;
                }
                let downloaded: u64 = sampler_progress
                    .iter()
                    .map(|slot| *slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
                    .sum();
                let delta = downloaded.saturating_sub(window_bytes);
                let remaining = file_size.saturating_sub(downloaded);
                if slow::is_slow(delta, elapsed, remaining).is_some() {
                    sampler_slow_abort.store(true, Ordering::Relaxed);
                    return;
                }
                window_started = Instant::now();
                window_bytes = downloaded;
            }
        }))
    } else {
        None
    });

    let start_time = Instant::now();
    let mut handles: Vec<ChunkHandle> = Vec::with_capacity(chunk_count + SPLIT_SLOTS);
    for (index, &(start, end)) in ranges.iter().enumerate() {
        handles.push(spawn_chunk_task(
            client,
            url,
            &format!("{}.part{}", local_path, index),
            index,
            start,
            end,
            &rate_limiter,
            &chunk_progress,
            &file_progress,
            &pause_flag,
            &cancel_flag,
            Some(slow_abort.clone()),
        ));
    }

    // 监督循环：定期让出并检查是否还有分片在跑。
    // 用 is_finished 而不是 join_next —— 拆分提速会 abort 原任务，被 abort 的任务不会
    // 返回结果，按完成计数会永久少一格，is_finished 没这个问题。
    let mut split: Option<SplitHalves> = None;
    loop {
        if handles.iter().all(|handle| handle.is_finished()) {
            break;
        }
        tokio::time::sleep(SPLIT_POLL_INTERVAL).await;
        if split.is_none() && !allow_slow_abort && slow_abort.load(Ordering::Relaxed) {
            split = try_split_straggler(
                &mut handles,
                local_path,
                &ranges,
                chunk_count,
                client,
                url,
                &rate_limiter,
                &chunk_progress,
                &pause_flag,
                &cancel_flag,
            );
        }
    }

    let mut all_ok = true;
    let mut total_downloaded: u64 = 0;
    let mut last_error = String::new();

    // 先收「拆分提速」的两半：两半不计 file_progress（避免与原任务重复计数），
    // 成功拼装后一次性补上；前缀字节原本就由被中止的原任务计过账了。
    let mut split_ok = false;
    if let Some(ref halves) = split {
        let mut halves_bytes = 0_u64;
        let mut halves_ok = true;
        for offset in 0..SPLIT_SLOTS {
            match handles[chunk_count + offset].await {
                Ok((_, Ok(bytes))) => halves_bytes += bytes,
                Ok((index, Err(e))) => {
                    halves_ok = false;
                    last_error = format!("chunk {} 拆分半段失败: {}", index, e);
                }
                Err(e) => {
                    halves_ok = false;
                    last_error = format!("chunk 拆分半段任务异常: {}", e);
                }
            }
        }
        let part_path = format!("{}.part{}", local_path, halves.index);
        let prefix = std::fs::metadata(&part_path)
            .map(|meta| meta.len())
            .unwrap_or(0);
        if halves_ok {
            match assemble_split(&part_path, &halves.left_path, &halves.right_path) {
                Ok(()) => {
                    split_ok = true;
                    total_downloaded += prefix + halves_bytes;
                    if let Some(ref fp) = file_progress {
                        add_progress(fp, halves_bytes);
                    }
                    log_debug!(
                        "[Chunk] 拆分分片 {} 拼装完成: {}",
                        halves.index,
                        format::bytes(prefix + halves_bytes)
                    );
                }
                Err(e) => {
                    halves_ok = false;
                    last_error = format!("拼装拆分分片失败: {}", e);
                }
            }
        }
        if !halves_ok {
            // 前缀已由原任务计入 file_progress，这里一并记入 total_downloaded，
            // 失败回滚时才不会漏账
            total_downloaded += prefix;
            log_warn!("[Chunk] {}", last_error);
            all_ok = false;
        }
    }

    for index in 0..chunk_count {
        // 拆分成功的那个分片，原任务是被主动 abort 的，结果无意义
        let aborted = split
            .as_ref()
            .map(|halves| halves.index == index)
            .unwrap_or(false)
            && split_ok;
        match handles[index].await {
            Ok((idx, Ok(bytes))) => {
                total_downloaded += bytes;
                log_debug!("[Chunk] chunk {} 完成: {}", idx, format::bytes(bytes));
            }
            Ok((idx, Err(e))) => {
                if !aborted {
                    all_ok = false;
                    last_error = format!("chunk {} 失败: {}", idx, e);
                    log_warn!("[Chunk] {}", last_error);
                }
            }
            Err(e) => {
                if !aborted {
                    all_ok = false;
                    last_error = format!("chunk task panic: {}", e);
                    log_warn!("[Chunk] {}", last_error);
                }
            }
        }
    }

    if !all_ok {
        // 采样器因低速置位时，失败原因要报「低速」而不是某个分片的普通错误：
        // 调用方据此换源，而不是在本 URL 上继续重试
        let slow = slow_abort.load(Ordering::Relaxed);
        // 断点续传：不清理 .part 文件，保留用于重试时续传
        // 回滚 file_progress：本次分片下载增量加的部分无效（文件将被重新下载），
        // 避免重试时 downloaded_bytes 持续累加导致进度偏高/超过 total
        if let Some(ref fp) = file_progress {
            let mut p = fp.lock().unwrap();
            p.downloaded_bytes = p.downloaded_bytes.saturating_sub(total_downloaded);
        }
        return ChunkDownloadResult {
            downloaded: total_downloaded,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            slow,
            error: Some(if slow {
                "线路持续低速，已中止当前源".to_string()
            } else {
                last_error
            }),
        };
    }

    // 合并分片到目标文件（合并前会校验每个 .part 大小匹配期望值，
    // 避免服务端提前断流导致的部分下载被误合并为损坏文件）
    if let Err(e) = merge_chunks(local_path, chunk_count, file_size) {
        // 断点续传：不清理 .part 文件，保留用于重试时续传
        // （合并失败通常是某分片大小不匹配，保留已下载部分可避免重头下载）
        // 回滚 file_progress：分片字节已全部计入，但合并失败文件将被重新下载，
        // 避免 downloaded_bytes 与后续单流/续传重复累计导致进度虚高
        if let Some(ref fp) = file_progress {
            let mut p = fp.lock().unwrap();
            p.downloaded_bytes = p.downloaded_bytes.saturating_sub(total_downloaded);
        }
        return ChunkDownloadResult {
            downloaded: total_downloaded,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some(format!("合并分片失败：{}", e)),
            slow: false,
        };
    }

    // 清理临时文件
    for i in 0..chunk_count {
        let _ = std::fs::remove_file(format!("{}.part{}", local_path, i));
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 {
        (total_downloaded as f64 / elapsed) as u64
    } else {
        0
    };

    log_debug!(
        "[Chunk] 分片下载完成: {} ({}, {:.1}s, {})",
        local_path,
        format::bytes(total_downloaded),
        elapsed,
        format::speed(speed)
    );

    ChunkDownloadResult {
        downloaded: total_downloaded,
        total: file_size,
        speed,
        status: DownloadStatus::Completed,
        error: None,
        slow: false,
    }
}

#[cfg(test)]
#[path = "api_test.rs"]
mod api_test;
