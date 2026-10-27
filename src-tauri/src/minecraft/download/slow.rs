//! 慢速线路判定
//!
//! 连接"假活"（能连上、有数据但吞吐极低）时，原有逻辑只能等 15s 无数据流动才失败，
//! 整包下载会被一条烂线路拖住。这里按「窗口内聚合吞吐」判定：持续 5s 低于 256 KiB/s
//! 且剩余数据够多，才认为线路不可用，交给重试层换源。
//!
//! 判慢只在「后面还有别的候选源」时启用（见 `retry`），避免把一条虽慢但可用的唯一
//! 线路判死。

use std::time::{Duration, Instant};

/// 采样窗口
pub(crate) const WINDOW: Duration = Duration::from_secs(5);

/// 窗口内平均吞吐下限（字节/秒）
pub(crate) const FLOOR_BYTES_PER_SEC: u64 = 256 * 1024;

/// 剩余数据不足该值时不再判慢：收尾阶段本来就会慢下来
pub(crate) const MIN_REMAINING: u64 = 1024 * 1024;

/// 判断窗口样本是否属于「持续低速」，命中时返回窗口内平均吞吐
///
/// - `delta_bytes`：本窗口内新增的字节数
/// - `elapsed`：本窗口已经过的时间
/// - `remaining`：文件剩余待下载字节数（未知时传 0，即不判慢）
pub(crate) fn is_slow(delta_bytes: u64, elapsed: Duration, remaining: u64) -> Option<u64> {
    if elapsed < WINDOW || remaining < MIN_REMAINING {
        return None;
    }
    let seconds = elapsed.as_secs_f64().max(0.001);
    let bytes_per_second = (delta_bytes as f64 / seconds) as u64;
    (bytes_per_second < FLOOR_BYTES_PER_SEC).then_some(bytes_per_second)
}

/// 滑动窗口探测器（单流下载用：由调用方在写入字节后调用 `add`）
pub(crate) struct SlowDetector {
    window_start: Instant,
    window_bytes: u64,
}

impl SlowDetector {
    pub(crate) fn new() -> Self {
        Self {
            window_start: Instant::now(),
            window_bytes: 0,
        }
    }

    /// 记录本窗口新增的字节数
    pub(crate) fn add(&mut self, bytes: u64) {
        self.window_bytes = self.window_bytes.saturating_add(bytes);
    }

    /// 窗口到期则结算并重置；命中低速时返回窗口内平均吞吐
    pub(crate) fn poll(&mut self, remaining: u64) -> Option<u64> {
        let elapsed = self.window_start.elapsed();
        let verdict = is_slow(self.window_bytes, elapsed, remaining);
        if elapsed >= WINDOW {
            self.window_start = Instant::now();
            self.window_bytes = 0;
        }
        verdict
    }
}

/// 线路持续低速：调用方应换源，而不是在同一 URL 上继续重试
#[derive(Debug)]
pub(crate) struct SlowRouteError {
    /// 触发判慢时的窗口平均吞吐（字节/秒）
    pub bytes_per_second: u64,
}

impl std::fmt::Display for SlowRouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "线路持续低速（{} KiB/s），已中止当前源",
            self.bytes_per_second / 1024
        )
    }
}

impl std::error::Error for SlowRouteError {}

#[cfg(test)]
#[path = "slow_test.rs"]
mod slow_test;
