//! 段表引擎：按块把多个源并行用起来
//!
//! 与「逐源重试」的区别在调度单位：这里以块为单位，哪个源快就让它多吃块，
//! 某条源断了它手上的块立刻能被别的源抢走，不需要把整个文件重下。
//! 位图落盘（`.partmap`）后，失败重试可复用已完成块，也能按「块的来源」定位可疑块。

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::{log_debug, log_warn};

use super::super::edge;
use super::super::rate_limiter::RateLimiter;
use super::super::types::GlobalProgress;
use super::block::{download_block, BlockError};
use super::table::SegmentTable;

/// 采样与调度间隔
const SAMPLE_INTERVAL: Duration = Duration::from_secs(2);

/// 单块最长耗时（块内已有 20s 无数据超时，这里是兜底）
const BLOCK_TIMEOUT: Duration = Duration::from_secs(300);

/// 单文件同时最多几条连接
pub(crate) const MAX_WORKERS_PER_FILE: usize = 12;

/// 连续多少个窗口没有进展就放弃
const IDLE_WINDOW_LIMIT: usize = 6;

/// 明显快于全局均值才给它加连接（避免抖动导致连接数乱涨）
const GROW_FACTOR: f64 = 1.2;

/// 初始 worker 分配：只有一条源就全给它，多条源先各给一条
pub(crate) fn initial_workers(source_count: usize, budget: usize) -> Vec<usize> {
    if source_count == 0 {
        return Vec::new();
    }
    let budget = budget.max(1);
    if source_count == 1 {
        return vec![budget];
    }
    let mut counts = vec![0_usize; source_count];
    for slot in counts.iter_mut().take(source_count.min(budget)) {
        *slot = 1;
    }
    counts
}

/// 是否该给某条源再加一条连接
pub(crate) fn should_grow(rate: u64, avg: u64, active_total: usize, budget: usize) -> bool {
    if active_total >= budget {
        return false;
    }
    if avg == 0 {
        return rate > 0;
    }
    (rate as f64) >= (avg as f64) * GROW_FACTOR
}

/// 引擎结果
#[derive(Debug)]
pub(crate) struct SegmentOutcome {
    pub downloaded: u64,
    pub total: u64,
    pub complete: bool,
    pub error: Option<String>,
}

/// 一条源
struct SourceSlot {
    url: String,
    /// 官方源可以领「被隔离」的块（镜像连续失败过的块不再交给它）
    official: bool,
    dead: AtomicBool,
    active: AtomicUsize,
    /// 本轮窗口内该源下载的字节（每轮清零）
    bytes: AtomicU64,
}

struct Engine {
    table: Arc<SegmentTable>,
    sources: Vec<Arc<SourceSlot>>,
    stop: AtomicBool,
    map_path: String,
}

struct WorkerArgs {
    engine: Arc<Engine>,
    /// 基础客户端；每条源每次取块时再按边缘优选换用「钉住 IP」的客户端
    base_client: reqwest::Client,
    path: String,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    file_progress: Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
}

fn cancel_requested(flag: &Option<Arc<AtomicBool>>) -> bool {
    flag.as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
}

/// 多源并行下载同一文件
///
/// `official_index` 指定的那条源可以领被隔离的块；其余源只领干净块。
/// `only_source` 为 Some 时只保留那一条源，并把其它源下过的块标回待领
/// ——内容校验失败时用它「只重下可疑块」而不是整份重下。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn download_multi(
    client: &reqwest::Client,
    urls: &[String],
    official_index: usize,
    only_source: Option<usize>,
    path: &str,
    file_size: u64,
    budget: usize,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    file_progress: Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
) -> SegmentOutcome {
    if urls.is_empty() || file_size == 0 {
        return SegmentOutcome {
            downloaded: 0,
            total: file_size,
            complete: false,
            error: Some("缺少可用源或文件大小未知".to_string()),
        };
    }
    let map_path = format!("{}.partmap", path);
    let data_len = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    let table = match SegmentTable::load(Path::new(&map_path), file_size, data_len) {
        Some(restored) => {
            log_debug!(
                "[Segment] 复用已完成块: {} 块 / 共 {} 字节",
                restored.done_count(),
                restored.done_bytes()
            );
            Arc::new(restored)
        }
        None => Arc::new(SegmentTable::new(file_size)),
    };
    if let Some(keep) = only_source {
        let invalidated = table.invalidate_others(keep);
        if invalidated > 0 {
            log_warn!(
                "[Segment] 内容校验失败，作废 {} 块并只用源 #{} 重下",
                invalidated,
                keep
            );
        }
    }
    let sources: Vec<Arc<SourceSlot>> = urls
        .iter()
        .enumerate()
        .map(|(index, url)| {
            let keep = only_source.is_none_or(|keep| keep == index);
            Arc::new(SourceSlot {
                url: url.clone(),
                official: index == official_index || urls.len() == 1 || only_source == Some(index),
                dead: AtomicBool::new(!keep),
                active: AtomicUsize::new(0),
                bytes: AtomicU64::new(0),
            })
        })
        .collect();
    let engine = Arc::new(Engine {
        table: table.clone(),
        sources: sources.clone(),
        stop: AtomicBool::new(false),
        map_path: map_path.clone(),
    });
    let args = Arc::new(WorkerArgs {
        engine: engine.clone(),
        base_client: client.clone(),
        path: path.to_string(),
        rate_limiter,
        file_progress,
        pause_flag,
        cancel_flag,
    });

    let budget = budget.clamp(1, MAX_WORKERS_PER_FILE);
    log_debug!(
        "[Segment] 并行下载: {} (块长 {}, 源 {} 条, 连接额度 {})",
        path,
        table.seg_len(),
        urls.len(),
        budget
    );
    let mut set: JoinSet<()> = JoinSet::new();
    for (index, count) in initial_workers(sources.len(), budget).iter().enumerate() {
        if sources[index].dead.load(Ordering::Relaxed) {
            continue;
        }
        for _ in 0..*count {
            sources[index].active.fetch_add(1, Ordering::Relaxed);
            spawn_worker(&mut set, &args, index);
        }
    }

    // 监督循环：采样 → 判死 → 扩容；顺手把位图落盘（几 KB，代价可忽略）
    let mut last_done = 0_u64;
    let mut idle_windows = 0_usize;
    loop {
        tokio::time::sleep(SAMPLE_INTERVAL).await;
        if cancel_requested(&args.cancel_flag) {
            engine.stop.store(true, Ordering::Relaxed);
            break;
        }
        if table.is_complete() {
            break;
        }
        if let Err(e) = table.save(Path::new(&engine.map_path)) {
            log_debug!("[Segment] 位图落盘失败: {}", e);
        }
        let done = table.done_bytes();
        if done == last_done {
            idle_windows += 1;
        } else {
            idle_windows = 0;
        }
        last_done = done;
        if idle_windows >= IDLE_WINDOW_LIMIT {
            log_warn!(
                "[Segment] {} 个窗口无进展，结束本次并行: {}",
                idle_windows,
                path
            );
            engine.stop.store(true, Ordering::Relaxed);
            break;
        }

        let window_secs = SAMPLE_INTERVAL.as_secs().max(1);
        let rates: Vec<u64> = sources
            .iter()
            .map(|slot| slot.bytes.swap(0, Ordering::Relaxed) / window_secs)
            .collect();
        let alive: Vec<usize> = sources
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.dead.load(Ordering::Relaxed))
            .map(|(index, _)| index)
            .collect();
        if alive.is_empty() {
            break;
        }
        let active_total: usize = sources
            .iter()
            .map(|slot| slot.active.load(Ordering::Relaxed))
            .sum();

        // 判死：本轮零产出且有别的源可用（唯一活源永远保留）
        if alive.len() > 1 && active_total > 1 {
            for index in alive.iter().copied() {
                if rates[index] == 0 {
                    sources[index].dead.store(true, Ordering::Relaxed);
                    log_warn!("[Segment] 源本轮零产出，停用: {}", sources[index].url);
                }
            }
        }

        // 扩容：给本轮最快且明显高于均值的源加一条连接
        let usable: Vec<usize> = sources
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.dead.load(Ordering::Relaxed))
            .map(|(index, _)| index)
            .collect();
        if usable.is_empty() {
            break;
        }
        let sum: u64 = usable.iter().map(|index| rates[*index]).sum();
        let avg = sum / usable.len().max(1) as u64;
        let best = usable.iter().copied().max_by_key(|index| rates[*index]);
        if let Some(best) = best {
            if should_grow(rates[best], avg, active_total, budget) {
                sources[best].active.fetch_add(1, Ordering::Relaxed);
                spawn_worker(&mut set, &args, best);
                log_debug!(
                    "[Segment] 扩容: {} (+1 连接, {} B/s, 均值 {} B/s)",
                    sources[best].url,
                    rates[best],
                    avg
                );
            }
        }
    }

    engine.stop.store(true, Ordering::Relaxed);
    while set.join_next().await.is_some() {}

    let downloaded = table.done_bytes();
    let complete = table.is_complete();
    if complete {
        // 完成即清理位图，避免下次误复用
        let _ = std::fs::remove_file(&engine.map_path);
    } else {
        // 失败也要落盘：下一轮（或下次启动）可以只补没完成的块
        if let Err(e) = table.save(Path::new(&engine.map_path)) {
            log_debug!("[Segment] 位图落盘失败: {}", e);
        }
    }
    SegmentOutcome {
        downloaded,
        total: file_size,
        complete,
        error: if complete {
            None
        } else {
            Some(format!(
                "并行下载未完成：{}/{} 字节（{}）",
                downloaded,
                file_size,
                if table.has_claimable() {
                    "仍有可领的块，但当前源都下不动"
                } else {
                    "剩余块已全部隔离，需要换源重试"
                }
            ))
        },
    }
}

fn spawn_worker(set: &mut JoinSet<()>, args: &Arc<WorkerArgs>, source_index: usize) {
    let args = Arc::clone(args);
    set.spawn(async move {
        run_worker(args, source_index).await;
    });
}

/// 单个 worker：认领 → 下载 → 记账，直到没有可领的块或被停用
async fn run_worker(args: Arc<WorkerArgs>, source_index: usize) {
    let engine = &args.engine;
    let slot = &engine.sources[source_index];
    while !engine.stop.load(Ordering::Relaxed) && !slot.dead.load(Ordering::Relaxed) {
        let Some(segment) = engine.table.claim(slot.official) else {
            if engine.table.is_complete() {
                break;
            }
            // 暂时无可领的块：可能都在飞，等一会儿再看
            tokio::time::sleep(Duration::from_millis(50)).await;
            continue;
        };
        // 每条源各自做边缘优选：同一条源拿到钉住 IP 的客户端，探测在后台完成
        let client = edge::client_for(&slot.url, &args.base_client);
        let outcome = tokio::time::timeout(
            BLOCK_TIMEOUT,
            download_block(
                &client,
                &slot.url,
                &args.path,
                segment,
                &args.rate_limiter,
                &args.file_progress,
                &slot.bytes,
                &args.pause_flag,
                &args.cancel_flag,
            ),
        )
        .await;
        match outcome {
            Ok(Ok(_)) => engine.table.complete(segment.index, source_index),
            Ok(Err(BlockError::RangeUnsupported)) => {
                log_debug!("[Segment] 源不支持 Range，停用: {}", slot.url);
                slot.dead.store(true, Ordering::Relaxed);
                engine.table.fail(segment.index);
                break;
            }
            Ok(Err(BlockError::Cancelled)) => {
                engine.table.fail(segment.index);
                break;
            }
            Ok(Err(e)) => {
                engine.table.fail(segment.index);
                // 传输层失败计入该源的边缘节点健康度
                edge::note_failure(&slot.url);
                log_debug!("[Segment] 块 {} 失败: {} ({})", segment.index, e, slot.url);
            }
            Err(_) => {
                engine.table.fail(segment.index);
                edge::note_failure(&slot.url);
                log_warn!("[Segment] 块 {} 超时: {}", segment.index, slot.url);
            }
        }
    }
    slot.active.fetch_sub(1, Ordering::Relaxed);
}

#[cfg(test)]
#[path = "multi_test.rs"]
mod multi_test;
