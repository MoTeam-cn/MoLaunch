//! build script 模块化拆分
//!
//! - mod.rs：模块入口
//! - cubiomes_wasm.rs：emcc 编译 cubiomes 到 WASM 的逻辑
//! - emsdk.rs：emcc 可执行文件查找与环境变量配置

pub mod cubiomes_wasm;
pub mod emsdk;
