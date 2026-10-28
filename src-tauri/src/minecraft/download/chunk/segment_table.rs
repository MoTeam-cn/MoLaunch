//! 段表引擎门面：开关 + 对外入口
//!
//! 开关默认开启；关掉后退回「逐源重试 + 分片下载」的旧实现，便于出问题时一键回退。

use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) use super::multi::{download_multi, SegmentOutcome};

/// 开关（`download.segment_table`）
static ENABLED: AtomicBool = AtomicBool::new(true);

/// 设置开关
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// 开关是否开启
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
