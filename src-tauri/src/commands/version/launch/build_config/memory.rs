//! 启动配置中的内存解析。

use crate::minecraft::version::setup::VersionSetup;
use crate::state::AppConfig;

/// 按版本独立内存模式解析最终最小/最大内存。
pub(super) fn resolve_memory(setup: &VersionSetup, config: &AppConfig) -> (u32, u32) {
    match setup.java.memory_mode.as_deref().filter(|s| !s.is_empty()) {
        Some("auto") => crate::minecraft::system::suggest_memory(),
        Some("custom") => {
            let max = setup.java.max_memory.unwrap_or(config.memory.max);
            let min = setup.java.min_memory.unwrap_or(max / 2);
            (min, max)
        }
        _ => (config.memory.min, config.memory.max),
    }
}
