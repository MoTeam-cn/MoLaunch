//! build script 模块化拆分
//!
//! - mod.rs：模块入口
//! - updater.rs：自动构建 updater.exe（Windows 便携版更新器，仅 Windows 平台）

#[cfg(target_os = "windows")]
pub mod updater;
