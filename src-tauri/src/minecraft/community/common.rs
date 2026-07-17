//! 社区模块公共工具函数
//!
//! 供 curseforge / modrinth / preload / sources 等模块复用。

use std::time::Instant;

/// 格式化耗时：< 1000ms 显示 ms，>= 1000ms 显示 s
///
/// 统一替代此前分散在 sources.rs / curseforge.rs / modrinth.rs / preload.rs 中的
/// 同名私有实现。
pub fn fmt_elapsed(start: Instant) -> String {
    let ms = start.elapsed().as_millis();
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}
