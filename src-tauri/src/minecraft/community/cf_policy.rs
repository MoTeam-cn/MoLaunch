//! 受限 CurseForge 文件地址推导的进程内开关
//!
//! 推导属于开发者选项（`download.cf_derive_url`）。社区资源的转换层是同步、
//! 无状态的纯函数，拿不到 `AppState` 的异步配置锁，因此把开关镜像成进程级
//! `AtomicBool`：启动加载配置与保存配置时各写一次，读取端零成本。

use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);

/// 写入开关（启动加载配置 / 保存配置时调用）
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// 当前是否允许推导受限文件的 CDN 地址（默认 false）
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
