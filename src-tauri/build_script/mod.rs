//! build script 模块化拆分
//!
//! - mod.rs：模块入口
//! - cubiomes_wasm.rs：emcc 编译 cubiomes 到 WASM 的逻辑
//! - emsdk.rs：emcc 可执行文件查找与环境变量配置
//! - updater.rs：自动构建 updater.exe（Windows 便携版更新器，仅 Windows 平台）
//! - easytier.rs：检查 easytier-core 嵌入式资源（按 CARGO_CFG_TARGET_OS/ARCH 定位，全平台生效）

pub mod cubiomes_wasm;
pub mod easytier;
pub mod emsdk;
#[cfg(target_os = "windows")]
pub mod updater;
