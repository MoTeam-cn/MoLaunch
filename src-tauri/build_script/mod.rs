//! build script 模块化拆分
//!
//! - mod.rs：模块入口
//! - updater.rs：自动构建 updater.exe（Windows 便携版更新器，仅 Windows 平台）
//! - easytier.rs：检查 easytier-core 嵌入式资源（按 CARGO_CFG_TARGET_OS/ARCH 定位，全平台生效）

pub mod easytier;
#[cfg(target_os = "windows")]
pub mod updater;
