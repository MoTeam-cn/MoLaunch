//! 清理游戏垃圾文件（扫描 `.minecraft` 下可清理内容，分三类）
//! 根目录固定子目录：logs/crash-reports/.mixin.out/assets/cache/.fabric/remapCache/screenshots；
//! 版本目录下子目录（版本隔离模式）：logs/crash-reports/.mixin.out/.fabric/processedMods/
//! remappedJars；原生库提取目录 `<ver>-natives`（非 natives）。
//! `execute` 删除时严格路径安全检查，仅删扫描阶段发现的目录，避免路径遍历攻击。

mod execute;
mod fs;
mod scan;

pub use execute::execute;
pub use scan::scan;