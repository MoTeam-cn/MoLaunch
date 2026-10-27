//! 公共镜像闸门：全局并发上限与限流冷却
//!
//! 公共镜像（BMCLAPI / MCIM / 自建 mocdn）对高并发 Range 请求很敏感：一次打几十条
//! 请求会被限流，随后整批下载回落到官方源，反而更慢。本模块只做两件事：
//! 1. 镜像域全局限并发：在途请求超过上限时排队等待；
//! 2. 收到 429/503 后按指数退避给该镜像设冷却，冷却期内调用方直接换源。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use reqwest::header::RETRY_AFTER;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// 第三方公共镜像（OpenBMCLAPI / MCIM）同时在途的请求上限
///
/// 这类镜像由社区维护、对高并发 Range 请求很敏感，打满后会被 429。
const MAX_IN_FLIGHT: usize = 4;

/// 自建 CDN（mocdn.net）同时在途的请求上限
///
/// 自建源容量可控，放宽一倍，避免成为正常下载的瓶颈。
const OWN_CDN_MAX_IN_FLIGHT: usize = 8;

/// 冷却退避起点（首次被限流）
const BACKOFF_BASE: Duration = Duration::from_secs(2);

/// 冷却退避上限
const BACKOFF_MAX: Duration = Duration::from_secs(30);

/// 退避步数封顶（2s → 4s → 8s → 16s → 30s）
const BACKOFF_MAX_STEP: u32 = 5;

/// 单次冷却最长等待：超过这个值就换源，不再原地等
pub(crate) const MAX_COOLDOWN_WAIT: Duration = Duration::from_secs(5);

/// 单个镜像主机的冷却状态
#[derive(Debug, Clone, Copy, Default)]
struct Cooldown {
    /// 冷却截止时间（None 表示当前未冷却）
    until: Option<Instant>,
    /// 连续被限流的次数，用于指数退避
    step: u32,
}

/// 每个镜像主机的在途上限
fn limit_for(host: &str) -> usize {
    if host.contains("mocdn") {
        OWN_CDN_MAX_IN_FLIGHT
    } else {
        MAX_IN_FLIGHT
    }
}

/// 取（或创建）某主机专属的信号量
///
/// 按主机分桶而不是一个全局桶：一个慢镜像不该占住另一个镜像的额度。
fn host_slots(host: &str) -> Arc<Semaphore> {
    static SLOTS: OnceLock<Mutex<HashMap<String, Arc<Semaphore>>>> = OnceLock::new();
    let slots = SLOTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = slots
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let entry = guard.entry(host.to_string());
    let bucket = entry.or_insert_with(|| Arc::new(Semaphore::new(limit_for(host))));
    bucket.clone()
}

fn cooldowns() -> &'static Mutex<HashMap<String, Cooldown>> {
    static COOLDOWNS: OnceLock<Mutex<HashMap<String, Cooldown>>> = OnceLock::new();
    COOLDOWNS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 镜像域名关键字（与 sources::is_mirror_url 保持一致）
const MIRROR_HOST_KEYWORDS: [&str; 3] = ["bmclapi", "mocdn", "mcimirror"];

/// URL 是否指向需要限流的公共镜像
///
/// 只按**主机名**判定：路径里恰好含 "bmclapi" 的普通站点不该被限流。
fn is_mirror(url: &str) -> bool {
    mirror_host(url).is_some()
}

/// 取出镜像主机名（小写）；不是镜像域时返回 None
fn mirror_host(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    let hit = MIRROR_HOST_KEYWORDS.iter().any(|k| host.contains(*k));
    hit.then_some(host)
}

/// 限流 key：按主机归并（同一镜像的不同路径共享槽位与冷却）
fn gate_key(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .unwrap_or_else(|| url.to_string())
}

/// 第 `step` 次被限流后的冷却时长（纯函数，便于单测）
fn backoff_for(step: u32) -> Duration {
    let step = step.clamp(1, BACKOFF_MAX_STEP);
    let factor = 1u32 << (step - 1);
    BACKOFF_BASE.saturating_mul(factor).min(BACKOFF_MAX)
}

/// 申请镜像请求槽位
///
/// - 非镜像 URL 直接返回 `None`（不参与限流）
/// - 镜像 URL 返回许可：调用方需在**整个请求（含响应体读取）**期间持有
pub(crate) async fn acquire(url: &str) -> Option<OwnedSemaphorePermit> {
    let host = mirror_host(url)?;
    host_slots(&host).acquire_owned().await.ok()
}

/// 查询镜像当前剩余冷却时长
pub(crate) fn cooldown_remaining(url: &str) -> Option<Duration> {
    if !is_mirror(url) {
        return None;
    }
    let key = gate_key(url);
    let now = Instant::now();
    let mut map = cooldowns().lock().ok()?;
    let entry = map.get_mut(&key)?;
    match entry.until {
        Some(until) if until > now => Some(until.duration_since(now)),
        Some(_) => {
            entry.until = None;
            None
        }
        None => None,
    }
}

/// 记录一次镜像响应：429/503 触发冷却，2xx 清除冷却
///
/// 返回本次设置的冷却时长（`None` 表示未触发冷却）
pub(crate) fn observe(url: &str, response: &reqwest::Response) -> Option<Duration> {
    if !is_mirror(url) {
        return None;
    }
    let key = gate_key(url);
    match response.status().as_u16() {
        200..=299 => {
            if let Ok(mut map) = cooldowns().lock() {
                map.remove(&key);
            }
            None
        }
        429 | 503 => {
            let hinted = retry_after(response);
            let mut map = cooldowns().lock().ok()?;
            let entry = map.entry(key).or_default();
            entry.step = entry.step.saturating_add(1);
            let cooldown = hinted.unwrap_or_else(|| backoff_for(entry.step));
            entry.until = Some(Instant::now() + cooldown);
            Some(cooldown)
        }
        _ => None,
    }
}

/// 解析 `Retry-After`（只认秒数形式，HTTP 日期形式忽略；上限 30s）
fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(|secs| Duration::from_secs(secs.min(30)))
}

#[cfg(test)]
#[path = "mirror_gate_test.rs"]
mod mirror_gate_test;
