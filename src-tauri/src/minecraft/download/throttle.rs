//! 全局限流压力信号
//!
//! 镜像闸门（`mirror_gate`）知道「这条镜像刚被 429/503」，会给它设冷却；
//! 但「本次下载整体该退一档」这件事原先没人接收——分块路径把闸门的返回值直接丢掉了。
//! 这里只做一件事：把限流事件攒成一个单调计数，监督循环每个窗口取一次快照，
//! 数量涨了就说明这一窗口又被限流过，于是本文件退一条连接、且这一窗口不扩容。
//!
//! 用单调快照而不是「取走清零」：多个文件并发下载时，谁取走都会把别人要看的信号抢掉。

use std::sync::atomic::{AtomicU64, Ordering};

static THROTTLES: AtomicU64 = AtomicU64::new(0);

/// 记一次限流（429 / 503）
pub(crate) fn note_throttle() {
    THROTTLES.fetch_add(1, Ordering::Relaxed);
}

/// 当前累计限流次数（单调递增，供各文件各自比较）
pub(crate) fn snapshot() -> u64 {
    THROTTLES.load(Ordering::Relaxed)
}

/// 单个下载任务的限流观察器：每个采样窗口问一次「上一窗口又被限流过吗」
///
/// 持有自己的基线，因此多个文件并发时互不干扰（谁也不会把别人的信号取走）。
pub(crate) struct ThrottleWatch {
    last: u64,
}

impl ThrottleWatch {
    pub(crate) fn new() -> Self {
        Self { last: snapshot() }
    }

    /// 上一窗口有新限流事件则返回 true（并把基线推到当前值）
    pub(crate) fn tripped(&mut self) -> bool {
        let now = snapshot();
        if now > self.last {
            self.last = now;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
#[path = "throttle_test.rs"]
mod throttle_test;
