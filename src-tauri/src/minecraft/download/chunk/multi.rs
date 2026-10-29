//! 段表引擎：按块把多个源并行用起来
//!
//! 调度决定见 `schedule`（纯函数），块与位图见 `table`，单块传输见 `block`。
//! 本文件只负责把它们串起来：起 worker、采样、按决策调整、落盘位图。

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::{log_debug, log_warn};

use super::super::budget;
use super::super::credit::ProgressCredit;
use super::super::edge;
use super::super::host_profile;
use super::super::rate_limiter::RateLimiter;
use super::block::{download_block, BlockError};
use super::schedule::{
    initial_workers, should_grow, should_retire, should_shrink, MAX_WORKERS_PER_FILE,
};
use super::table::SegmentTable;

/// 采样与调度间隔
const SAMPLE_INTERVAL: Duration = Duration::from_secs(2);

/// 单块最长耗时（块内已有 20s 无数据超时，这里是兜底）
const BLOCK_TIMEOUT: Duration = Duration::from_secs(300);

/// 连续多少个窗口没有进展就放弃
const IDLE_WINDOW_LIMIT: usize = 6;

/// 重跑范围：内容校验失败后逐级缩小可疑范围
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RetryScope {
    /// 正常下载（位图里有进度就接着用）
    Full,
    /// 只保留指定源下的块，其余作废——只重下可疑块
    Suspects(usize),
    /// 全部作废，只用指定源从头重下
    Everything(usize),
}

/// 引擎结果
#[derive(Debug)]
pub(crate) struct SegmentOutcome {
    /// 当前已完成（可信）的字节数
    pub downloaded: u64,
    pub total: u64,
    pub complete: bool,
    /// 未完成但已有进度：半成品值得保留，下次可续
    pub resumable: bool,
    pub error: Option<String>,
}

/// 一条源
struct SourceSlot {
    url: String,
    /// 官方源可以领「被隔离」的块（镜像连续失败过的块不再交给它）
    official: bool,
    dead: AtomicBool,
    active: AtomicUsize,
    /// 目标连接数：worker 在每次认领前发现 active > target 就自行退出（AIMD 收缩）
    target: AtomicUsize,
    /// 本轮窗口内该源下载的字节（每轮清零）
    bytes: AtomicU64,
    /// 全程累计字节（用于结束时上报吞吐给主机记忆）
    total_bytes: AtomicU64,
}

struct Engine {
    table: Arc<SegmentTable>,
    sources: Vec<Arc<SourceSlot>>,
    stop: AtomicBool,
    map_path: String,
    resume_key: Option<u64>,
}

struct WorkerArgs {
    engine: Arc<Engine>,
    /// 基础客户端；每条源每次取块时再按边缘优选换用「钉住 IP」的客户端
    base_client: reqwest::Client,
    path: String,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
}

fn cancel_requested(flag: &Option<Arc<AtomicBool>>) -> bool {
    flag.as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
}

/// 多源并行下载同一文件
///
/// `official_index` 指定的源可以领被隔离的块；`scope` 控制重跑时作废哪些已完成块；
/// `resume_key` 是「同一次下载」的指纹，防止同名但内容变了的文件复用旧位图；
/// 为 `None` 表示这次下载不允许续传（没有期望哈希，拼出来的内容无从裁决）。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn download_multi(
    client: &reqwest::Client,
    urls: &[String],
    official_index: usize,
    scope: RetryScope,
    resume_key: Option<u64>,
    path: &str,
    file_size: u64,
    budget: usize,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    credit: Option<&ProgressCredit<'_>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
) -> SegmentOutcome {
    // 占用一个活跃文件名额：文件下完（或被取消）自动归还，额度让给还在下的文件
    let _lease = budget::FileLease::open();
    let started_at = std::time::Instant::now();
    if urls.is_empty() || file_size == 0 {
        return SegmentOutcome {
            downloaded: 0,
            total: file_size,
            complete: false,
            resumable: false,
            error: Some("缺少可用源或文件大小未知".to_string()),
        };
    }
    let map_path = format!("{}.partmap", path);
    let data_len = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    let data_path = path.to_string();
    let load_map_path = map_path.clone();
    let restored = match resume_key {
        Some(key) => tokio::task::spawn_blocking(move || {
            SegmentTable::load(
                Path::new(&load_map_path),
                file_size,
                data_len,
                key,
                Some(Path::new(&data_path)),
            )
        })
        .await
        .ok()
        .flatten(),
        None => {
            // 没有期望哈希就不续传：旧位图留着也没人会信它，直接清掉
            let _ = std::fs::remove_file(&load_map_path);
            None
        }
    };
    let table = match restored {
        Some(restored) => {
            log_debug!(
                "[Segment] 复用已完成块: {} 块 / {} 字节",
                restored.done_count(),
                restored.done_bytes()
            );
            Arc::new(restored)
        }
        None => Arc::new(SegmentTable::new(file_size)),
    };
    let (keep_source, invalidated) = match scope {
        RetryScope::Full => (None, 0),
        RetryScope::Suspects(keep) => (Some(keep), table.invalidate_others(keep)),
        RetryScope::Everything(keep) => (Some(keep), table.invalidate_all()),
    };
    if invalidated > 0 {
        log_warn!(
            "[Segment] 内容校验失败，作废 {} 块后只用源 #{} 重下",
            invalidated,
            keep_source.unwrap_or(0)
        );
    }
    let sources: Vec<Arc<SourceSlot>> = urls
        .iter()
        .enumerate()
        .map(|(index, url)| {
            let usable = keep_source.is_none_or(|keep| keep == index);
            Arc::new(SourceSlot {
                url: url.clone(),
                official: index == official_index || urls.len() == 1 || keep_source == Some(index),
                dead: AtomicBool::new(!usable),
                active: AtomicUsize::new(0),
                target: AtomicUsize::new(0),
                bytes: AtomicU64::new(0),
                total_bytes: AtomicU64::new(0),
            })
        })
        .collect();
    let engine = Arc::new(Engine {
        table: table.clone(),
        sources: sources.clone(),
        stop: AtomicBool::new(false),
        map_path: map_path.clone(),
        resume_key,
    });
    let args = Arc::new(WorkerArgs {
        engine: engine.clone(),
        base_client: client.clone(),
        path: path.to_string(),
        rate_limiter,
        pause_flag,
        cancel_flag,
    });

    let ceiling = budget.clamp(1, MAX_WORKERS_PER_FILE);
    let initial_budget = budget::per_file_share(ceiling);
    log_debug!(
        "[Segment] 并行下载: {} (块长 {}, 源 {} 条, 连接额度 {}/{})",
        path,
        table.seg_len(),
        urls.len(),
        initial_budget,
        ceiling
    );
    let mut set: JoinSet<()> = JoinSet::new();
    // 只给活着的源分配 worker：定向重跑时保留下来的源不一定排在候选列表前几位，
    // 按位置分配会让额度全落到被停用的源上，真正能用的那条一个 worker 也拿不到
    let alive_indices: Vec<usize> = (0..sources.len())
        .filter(|index| !sources[*index].dead.load(Ordering::Relaxed))
        .collect();
    let counts = initial_workers(alive_indices.len(), initial_budget);
    for (index, count) in alive_indices.iter().zip(counts.iter()) {
        let mut count = *count;
        // 主机记忆：记得这台 CDN 适合几条流，就从几条起步（不超过本轮份额）
        if let Some(remembered) = host_profile::suggest(&sources[*index].url) {
            count = count.max(remembered.min(initial_budget));
        }
        sources[*index].target.store(count, Ordering::Relaxed);
        for _ in 0..count {
            sources[*index].active.fetch_add(1, Ordering::Relaxed);
            spawn_worker(&mut set, &args, *index);
        }
    }

    // 监督循环：采样 → 判死 → 收缩/扩容；顺手把位图落盘（几 KB，代价可忽略）
    let mut last_moved = 0_u64;
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
        save_map(&engine);
        if let Some(credit) = credit {
            credit.credit(table.done_bytes());
        }

        let alive: Vec<usize> = sources
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.dead.load(Ordering::Relaxed))
            .map(|(index, _)| index)
            .collect();
        if alive.is_empty() {
            break;
        }
        let window_secs = SAMPLE_INTERVAL.as_secs().max(1);
        let rates: Vec<u64> = sources
            .iter()
            .map(|slot| {
                let window = slot.bytes.swap(0, Ordering::Relaxed);
                slot.total_bytes.fetch_add(window, Ordering::Relaxed);
                window / window_secs
            })
            .collect();

        // 判活按「字节有没有在动」算，而不是「有没有块下完」：
        // 慢源下完一块可能要几十秒，按块算会把还在干活但速度慢的链接判死
        let moved: u64 = sources
            .iter()
            .map(|slot| slot.total_bytes.load(Ordering::Relaxed))
            .sum();
        if moved == last_moved {
            idle_windows += 1;
        } else {
            idle_windows = 0;
        }
        last_moved = moved;
        if idle_windows >= IDLE_WINDOW_LIMIT {
            log_warn!(
                "[Segment] {} 个窗口没有任何字节流动，结束本次并行: {}",
                idle_windows,
                path
            );
            engine.stop.store(true, Ordering::Relaxed);
            break;
        }
        let active_total: usize = sources
            .iter()
            .map(|slot| slot.active.load(Ordering::Relaxed))
            .sum();

        // 判死：本轮零产出且有别的源可用（唯一活源永远保留）
        for index in alive.iter().copied() {
            // 没分到 worker 的源不是「零产出」：它只是还没轮到，判死就再也不会被试用
            if sources[index].active.load(Ordering::Relaxed) == 0 {
                continue;
            }
            if should_retire(rates[index], alive.len(), active_total) {
                sources[index].dead.store(true, Ordering::Relaxed);
                log_warn!("[Segment] 源本轮零产出，停用: {}", sources[index].url);
            }
        }
        let usable: Vec<usize> = sources
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.dead.load(Ordering::Relaxed))
            .map(|(index, _)| index)
            .collect();
        if usable.is_empty() {
            break;
        }
        // 候选源比连接额度多时，排在后面的源一开始分不到 worker；
        // 前面的源都被判死后这里唤醒一条，否则「没人试过」的源永远没机会
        let still_active: usize = sources
            .iter()
            .map(|slot| slot.active.load(Ordering::Relaxed))
            .sum();
        if still_active == 0 {
            let index = usable[0];
            sources[index].target.store(1, Ordering::Relaxed);
            sources[index].active.fetch_add(1, Ordering::Relaxed);
            spawn_worker(&mut set, &args, index);
            log_debug!("[Segment] 唤醒未试过的源: {}", sources[index].url);
        }
        let sum: u64 = usable.iter().map(|index| rates[*index]).sum();
        let avg = sum / usable.len().max(1) as u64;
        // 每轮重算份额：别的文件下完会让这里的额度变大，反之变小
        let share = budget::per_file_share(ceiling);
        let total_target: usize = usable
            .iter()
            .map(|index| sources[*index].target.load(Ordering::Relaxed))
            .sum();

        // 收缩：明显低于均值的源减一条；worker 在下次认领前自行退出
        for index in usable.iter().copied() {
            let current = sources[index].target.load(Ordering::Relaxed);
            if current > 1 && (should_shrink(rates[index], avg, current) || total_target > share) {
                // 明显慢，或本文件已超出当前份额（别的文件也需要额度）
                sources[index].target.store(current - 1, Ordering::Relaxed);
                log_debug!(
                    "[Segment] 收缩: {} (-1 连接, {} B/s, 均值 {} B/s)",
                    sources[index].url,
                    rates[index],
                    avg
                );
            }
        }

        // 扩容：给本轮最快且明显高于均值的源加一条连接
        if let Some(best) = usable.iter().copied().max_by_key(|index| rates[*index]) {
            let current = sources[best].target.load(Ordering::Relaxed);
            if should_grow(rates[best], avg, total_target, share) {
                sources[best].target.store(current + 1, Ordering::Relaxed);
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

    // 把每条源这次学到的结论留给下一个文件
    let elapsed_secs = started_at.elapsed().as_secs_f64().max(0.001);
    for slot in sources.iter() {
        let bytes = slot.total_bytes.load(Ordering::Relaxed);
        let target = slot.target.load(Ordering::Relaxed).max(1);
        host_profile::observe(&slot.url, target, (bytes as f64 / elapsed_secs) as u64);
    }

    let downloaded = table.done_bytes();
    let complete = table.is_complete();
    if let Some(credit) = credit {
        credit.credit(downloaded);
    }
    if complete {
        // 数据先落盘再删账本：账本没了而数据还在页缓存时，掉电后文件会变成无法自证的半截
        if let Err(e) = flush_data_file(path) {
            log_warn!("[Segment] 数据落盘失败: {}", e);
        }
        // 完成即清理位图，避免下次误复用
        let _ = std::fs::remove_file(&engine.map_path);
    } else {
        // 未完成也落盘：下一轮（或下次启动）只补没完成的块
        save_map(&engine);
    }
    SegmentOutcome {
        downloaded,
        total: file_size,
        complete,
        resumable: !complete && downloaded > 0,
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

/// 把数据文件刷到稳定存储。
///
/// 必须用写句柄打开：Windows 的 FlushFileBuffers 要求句柄带写权限，
/// 只读句柄的 sync_all 会直接失败——那样等于没刷。
fn flush_data_file(path: &str) -> std::io::Result<()> {
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.sync_all()
}

fn save_map(engine: &Engine) {
    let Some(key) = engine.resume_key else {
        // 不允许续传的下载不留位图：下次也不会加载它
        return;
    };
    if let Err(e) = engine.table.save(Path::new(&engine.map_path), key) {
        log_debug!("[Segment] 位图落盘失败: {}", e);
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
        // AIMD 收缩：超出目标连接数的那几条在认领前退出，不打断进行中的块
        if slot.active.load(Ordering::Relaxed) > slot.target.load(Ordering::Relaxed) {
            break;
        }
        let Some(segment) = engine.table.claim(slot.official) else {
            if engine.table.is_complete() {
                break;
            }
            // 暂时无可领的块：可能都在飞，等一会儿再看
            tokio::time::sleep(Duration::from_millis(50)).await;
            continue;
        };
        // 每条源各自做边缘优选：命中缓存就用钉住 IP 的客户端，探测在后台完成
        let client = edge::client_for(&slot.url, &args.base_client);
        let outcome = tokio::time::timeout(
            BLOCK_TIMEOUT,
            download_block(
                &client,
                &slot.url,
                &args.path,
                segment,
                &args.rate_limiter,
                &slot.bytes,
                &args.pause_flag,
                &args.cancel_flag,
            ),
        )
        .await;
        match outcome {
            Ok(Ok((hash, _bytes))) => engine.table.complete(segment.index, source_index, hash),
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
                log_warn!("[Segment] 超时: 块 {} ({})", segment.index, slot.url);
            }
        }
    }
    slot.active.fetch_sub(1, Ordering::Relaxed);
}

#[cfg(test)]
#[path = "multi_test.rs"]
mod multi_test;
