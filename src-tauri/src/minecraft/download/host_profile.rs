//! 主机脾气记忆：同一主机学到的并发与吞吐，在文件之间传递
//!
//! 单文件内的 AIMD 学到的结论（这台 CDN 给几条流最舒服）在下完那一刻就丢了，装 300 个模组
//! 时每个文件都要重新摸一遍。这里把结论留下来，下一个文件直接从上次的结论起步；
//! 起点偏高也没关系——引擎每 2 秒采样一次，方向不对两个窗口内就会自己收回来。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// 记忆有效期：太久以前的经验不再可信（CDN 会扩容/调度）
const TTL: Duration = Duration::from_secs(30 * 60);

/// 吞吐 EMA 的新样本权重
const EMA_WEIGHT: f64 = 0.3;

/// 记忆里的并发目标上下限
const MIN_TARGET: usize = 1;
const MAX_TARGET: usize = 8;

/// 一条主机的档案
#[derive(Clone, Copy, Debug)]
struct Profile {
    target: usize,
    throughput: u64,
    learned_at: Instant,
}

fn profiles() -> &'static Mutex<HashMap<String, Profile>> {
    static PROFILES: OnceLock<Mutex<HashMap<String, Profile>>> = OnceLock::new();
    PROFILES.get_or_init(|| Mutex::new(HashMap::new()))
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
        },
        None => Profile {
            target,
            throughput,
            learned_at: now,
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
    let profiles = profiles().lock().unwrap_or_else(|p| p.into_inner());
    let profile = profiles.get(&host)?;
    if profile.learned_at.elapsed() > TTL {
        return None;
    }
    Some(profile.target.clamp(MIN_TARGET, MAX_TARGET))
}

/// 记录一次观测（引擎结束时按每条源调用）
pub(crate) fn observe(url: &str, target: usize, throughput: u64) {
    let Some(host) = host_of(url) else {
        return;
    };
    let now = Instant::now();
    let mut profiles = profiles().lock().unwrap_or_else(|p| p.into_inner());
    let previous = profiles
        .get(&host)
        .copied()
        .filter(|profile| profile.learned_at.elapsed() <= TTL);
    profiles.insert(host, merge(previous, target, throughput, now));
}

#[cfg(test)]
#[path = "host_profile_test.rs"]
mod host_profile_test;
