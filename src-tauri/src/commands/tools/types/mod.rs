//! 工具模块的统一类型定义
//!
//! 按 `commands/tools/` 子模块一一对应分组：
//! download/filename/cleanup/memory/mod_tools/data_export/crash_analyzer/screenshot/
//! resourcepack/version_json/archive/network/nbt/picker_window。
//! 注：原 `ToolsRequest` 已替换为通用的 `utils::dispatcher::ActionRequest`，
//! 与 `meta_manager` 共用同一请求体结构。

mod archive;
mod cleanup;
mod crash_analyzer;
mod data_export;
mod download;
mod filename;
mod memory;
mod mod_tools;
mod nbt;
mod network;
mod picker_window;
mod resourcepack;
mod screenshot;
mod version_json;

pub use archive::*;
pub use cleanup::*;
pub use crash_analyzer::*;
pub use data_export::*;
pub use download::*;
pub use filename::*;
pub use memory::*;
pub use mod_tools::*;
pub use nbt::*;
pub use network::*;
pub use picker_window::*;
pub use resourcepack::*;
pub use screenshot::*;
pub use version_json::*;

// 注：种子地图相关类型已删除——工具迁移至前端 WASM 方案，不再走后端 IPC。
// 前端通过 res:// 协议加载 cubiomes.wasm，在 Worker 中直接调用 cubiomes C 函数。
