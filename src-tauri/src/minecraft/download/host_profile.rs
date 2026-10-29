//! 主机脾气记忆：同一主机学到的并发与吞吐，在文件之间、进程之间传递
//!
//! 单文件内的 AIMD 学到的结论（这台 CDN 给几条流最舒服）在下完那一刻就丢了，装 300 个模组
//! 时每个文件都要重新摸一遍。这里把结论留下来，下一个文件直接从上次的结论起步；
//! 起点偏高也没关系——引擎每 2 秒采样一次，方向不对两个窗口内就会自己收回来。
//!
//! 记忆会落盘（`cache/download/host_profiles.json`），重启后仍然有效；条目带时间戳，
//! 超过 TTL 自动作废，条数上限 64，格式带版本号，读不懂就当成没有——不猜、不迁移。

use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{log_debug, log_warn};

/// 记忆有效期：太久以前的经验不再可信（CDN 会扩容/调度）
const TTL: Duration = Duration::from_secs(30 * 60);

/// 吞吐 EMA 的新样本权重
const EMA_WEIGHT: f64 = 0.3;

/// 记忆里的并发目标上下限
const MIN_TARGET: usize = 1;
const MAX_TARGET: usize = 8;

/// 落盘文件（相对缓存根目录）
const STORE_PATH: &str = "download/host_profiles.json";

/// 落盘格式版本
const STORE_VERSION: u32 = 1;

/// 落盘条目上限（超出按最近使用淘汰）
const MAX_ENTRIES: usize = 64;

/// 两次落盘的最小间隔，避免每个文件结束都写一次
const SAVE_INTERVAL: Duration = Duration::from_secs(60);

/// 记忆文件大小上限：正常只有几十 KB，突然变大说明被写坏了，别把内存交给它
const STORE_MAX_BYTES: u64 = 1024 * 1024;

/// 一条主机的档案
#[derive(Clone, Copy, Debug)]
struct Profile {
    target: usize,
    throughput: u64,
    learned_at: Instant,
    /// 这台主机是否支持 Range；None 表示没测过（当支持处理）
    supports_range: Option<bool>,
}

/// 运行时状态
struct State {
    profiles: HashMap<String, Profile>,
    last_save: Instant,
}

fn state() -> &'static Mutex<Option<State>> {
    static STATE: OnceLock<Mutex<Option<State>>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(None))
}

/// 从 URL 取主机名（小写）；解析失败返回 None
pub(crate) fn host_of(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()?
        .host_str()
        .map(str::to_ascii_lowercase)
}

/// 合并两次观测：并发目标取大（只是起点，偏了会被引擎纠正），吞吐做 EMA
fn merge(previous: Option<Profile>, target: usize, throughput: u64, now: Instant) -> Profile {
    let target = target.clamp(MIN_TARGET, MAX_TARGET);
    match previous {
        Some(previous) => Profile {
            target: previous.target.max(target).clamp(MIN_TARGET, MAX_TARGET),
            throughput: ema(previous.throughput, throughput),
            learned_at: now,
            // Range 能力与并发无关，学并发不能把这条结论抹掉
            supports_range: previous.supports_range,
        },
        None => Profile {
            target,
            throughput,
            learned_at: now,
            supports_range: None,
        },
    }
}

/// 指数滑动平均：previous 为 0（还没样本）时直接采用新值
fn ema(previous: u64, sample: u64) -> u64 {
    if previous == 0 {
        return sample;
    }
    let blended = previous as f64 * (1.0 - EMA_WEIGHT) + sample as f64 * EMA_WEIGHT;
    blended.round().max(0.0) as u64
}

/// 上次学到的并发目标（TTL 内有效）
pub(crate) fn suggest(url: &str) -> Option<usize> {
    let host = host_of(url)?;
    let mut guard = state().lock().unwrap_or_else(|p| p.into_inner());
    ensure_loaded(&mut guard);
    let profiles = &guard.as_ref()?.profiles;
    let profile = profiles.get(&host)?;
    if profile.learned_at.elapsed() > TTL {
        return None;
    }
    Some(profile.target.clamp(MIN_TARGET, MAX_TARGET))
}

/// 这台主机是否已被记为不支持 Range
///
/// 未知一律当「支持」：这条记忆只用来省掉注定白费的请求，不该拦下可能成功的。
pub(crate) fn range_unsupported(url: &str) -> bool {
    let Some(host) = host_of(url) else {
        return false;
    };
    let mut guard = state().lock().unwrap_or_else(|p| p.into_inner());
    ensure_loaded(&mut guard);
    guard
        .as_ref()
        .and_then(|state| state.profiles.get(&host))
        .filter(|profile| profile.learned_at.elapsed() <= TTL)
        .map(|profile| profile.supports_range == Some(false))
        .unwrap_or(false)
}

/// 记下「这台主机不支持 Range」
///
/// 只改这一个结论：学到的并发与吞吐是另一件事，不能被覆盖掉。
pub(crate) fn note_range_unsupported(url: &str) {
    let Some(host) = host_of(url) else {
        return;
    };
    let now = Instant::now();
    let mut guard = state().lock().unwrap_or_else(|p| p.into_inner());
    ensure_loaded(&mut guard);
    let Some(state) = guard.as_mut() else {
        return;
    };
    let previous = state
        .profiles
        .get(&host)
        .copied()
        .filter(|profile| profile.learned_at.elapsed() <= TTL);
    let mut profile = previous.unwrap_or(Profile {
        target: MIN_TARGET,
        throughput: 0,
        learned_at: now,
        supports_range: None,
    });
    profile.supports_range = Some(false);
    // 刷新时间：这条结论比一条并发的经验更重要，别让它先过期
    profile.learned_at = now;
    state.profiles.insert(host, profile);
    if let Some(snapshot) = due_snapshot(state, now) {
        drop(guard);
        if let Some(path) = store_path() {
            if let Err(e) = save_to(&path, &snapshot) {
                log_debug!("[HostProfile] 落盘失败: {}", e);
            }
        }
    }
}

/// 到点就落盘：返回要写的快照，未到间隔返回 None
fn due_snapshot(state: &mut State, now: Instant) -> Option<Store> {
    if state.last_save.elapsed() < SAVE_INTERVAL {
        return None;
    }
    state.last_save = now;
    prune(state);
    Some(snapshot(state))
}

/// 记录一次观测（引擎结束时按每条源调用）
pub(crate) fn observe(url: &str, target: usize, throughput: u64) {
    let Some(host) = host_of(url) else {
        return;
    };
    // 一个字节都没下成（比如整条源直接 404）不构成经验
    if throughput == 0 {
        return;
    }
    let now = Instant::now();
    let mut guard = state().lock().unwrap_or_else(|p| p.into_inner());
    ensure_loaded(&mut guard);
    let Some(state) = guard.as_mut() else {
        return;
    };
    let previous = state
        .profiles
        .get(&host)
        .copied()
        .filter(|profile| profile.learned_at.elapsed() <= TTL);
    state
        .profiles
        .insert(host, merge(previous, target, throughput, now));
    if let Some(snapshot) = due_snapshot(state, now) {
        drop(guard);
        if let Some(path) = store_path() {
            if let Err(e) = save_to(&path, &snapshot) {
                log_debug!("[HostProfile] 落盘失败: {}", e);
            }
        }
    }
}

/// 丢掉过期条目，并把条数压到上限以内（保留最近学到的）
fn prune(state: &mut State) {
    state
        .profiles
        .retain(|_, profile| profile.learned_at.elapsed() <= TTL);
    if state.profiles.len() <= MAX_ENTRIES {
        return;
    }
    let mut order: Vec<(String, Instant)> = state
        .profiles
        .iter()
        .map(|(host, profile)| (host.clone(), profile.learned_at))
        .collect();
    order.sort_by_key(|entry| Reverse(entry.1));
    let keep: Vec<String> = order
        .into_iter()
        .take(MAX_ENTRIES)
        .map(|(h, _)| h)
        .collect();
    state.profiles.retain(|host, _| keep.contains(host));
}

/// 落盘用的快照（时间换成绝对时间戳，重启后才能算 TTL）
fn snapshot(state: &State) -> Store {
    let now = unix_millis();
    let profiles = state
        .profiles
        .iter()
        .map(|(host, profile)| {
            let elapsed = profile.learned_at.elapsed().as_millis() as u64;
            (
                host.clone(),
                PersistedProfile {
                    target: profile.target,
                    throughput: profile.throughput,
                    supports_range: profile.supports_range,
                    learned_epoch_ms: now.saturating_sub(elapsed),
                },
            )
        })
        .collect();
    Store {
        version: STORE_VERSION,
        profiles,
    }
}

fn ensure_loaded(guard: &mut Option<State>) {
    if guard.is_some() {
        return;
    }
    let profiles = store_path()
        .as_deref()
        .and_then(|path| load_from(path).ok())
        .map(|store| store.into_runtime())
        .unwrap_or_default();
    if !profiles.is_empty() {
        log_debug!("[HostProfile] 读回 {} 条主机记忆", profiles.len());
    }
    *guard = Some(State {
        profiles,
        last_save: Instant::now(),
    });
}

fn store_path() -> Option<std::path::PathBuf> {
    Some(crate::storage::cache::Cache::instance().path(STORE_PATH))
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 落盘格式
#[derive(Debug, Default, Serialize, Deserialize)]
struct Store {
    version: u32,
    profiles: HashMap<String, PersistedProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedProfile {
    target: usize,
    throughput: u64,
    /// 旧文件里没有这个字段：缺省即「没测过」，不猜
    #[serde(default)]
    supports_range: Option<bool>,
    /// 学到这条经验的绝对时间（毫秒时间戳），用来跨进程算 TTL
    learned_epoch_ms: u64,
}

impl Store {
    /// 转成运行时表示：丢掉版本不符、过期、明显异常的条目
    fn into_runtime(self) -> HashMap<String, Profile> {
        if self.version != STORE_VERSION {
            log_warn!("[HostProfile] 记忆格式版本 {} 不认识，忽略", self.version);
            return HashMap::new();
        }
        let now = unix_millis();
        self.profiles
            .into_iter()
            .filter_map(|(host, stored)| {
                if stored.learned_epoch_ms > now {
                    return None;
                }
                let age = Duration::from_millis(now - stored.learned_epoch_ms);
                if age > TTL || stored.target == 0 {
                    return None;
                }
                Some((
                    host,
                    Profile {
                        target: stored.target.clamp(MIN_TARGET, MAX_TARGET),
                        throughput: stored.throughput,
                        supports_range: stored.supports_range,
                        learned_at: Instant::now().checked_sub(age).unwrap_or_else(Instant::now),
                    },
                ))
            })
            .collect()
    }
}

/// 读盘（测试用可指定路径）
fn load_from(path: &Path) -> anyhow::Result<Store> {
    let meta = std::fs::metadata(path)?;
    if meta.len() > STORE_MAX_BYTES {
        anyhow::bail!("主机记忆文件过大（{} 字节），忽略", meta.len());
    }
    let text = std::fs::read_to_string(path)?;
    let store: Store = serde_json::from_str(&text)?;
    Ok(store)
}

/// 写盘（测试用可指定路径）
fn save_to(path: &Path, store: &Store) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string(store)?;
    std::fs::write(path, text)?;
    Ok(())
}

#[cfg(test)]
#[path = "host_profile_test.rs"]
mod host_profile_test;
