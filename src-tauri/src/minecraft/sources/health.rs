//! 下载源健康度
//!
//! 官方源超时一次后，接下来一段时间内所有请求先走镜像。
//! 没有这层记忆的话，每个文件都要先等一次超时才回退——
//! 官方不稳定时体感就是"每个文件都卡一下"。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

/// 官方源判定为慢之后，多久再回头试它
const OFFICIAL_SLOW_TTL_SECS: u64 = 300;

/// 记到"某个时刻为止都别用官方"，0 表示官方正常
static OFFICIAL_SLOW_UNTIL: AtomicU64 = AtomicU64::new(0);
static START: OnceLock<Instant> = OnceLock::new();

fn now_secs() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_secs()
}

/// 纯逻辑：给定"慢到什么时候"和"现在几点"，判断官方是否仍在观察期
fn still_slow(slow_until: u64, now: u64) -> bool {
    slow_until > now
}

/// 标记官方源慢
pub fn mark_official_slow() {
    let until = now_secs() + OFFICIAL_SLOW_TTL_SECS;
    OFFICIAL_SLOW_UNTIL.store(until, Ordering::Relaxed);
}

/// 标记官方源恢复正常
pub fn mark_official_ok() {
    OFFICIAL_SLOW_UNTIL.store(0, Ordering::Relaxed);
}

/// 官方源当前是否处于"慢"的观察期
pub fn official_is_slow() -> bool {
    still_slow(OFFICIAL_SLOW_UNTIL.load(Ordering::Relaxed), now_secs())
}

/// 官方源观察期剩余秒数（0 表示正常），供日志展示
pub fn official_slow_remaining_secs() -> u64 {
    let until = OFFICIAL_SLOW_UNTIL.load(Ordering::Relaxed);
    until.saturating_sub(now_secs())
}

#[cfg(test)]
#[path = "health_test.rs"]
mod health_test;
