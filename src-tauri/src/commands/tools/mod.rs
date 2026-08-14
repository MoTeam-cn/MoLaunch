//! 工具模块（统一 IPC 入口）
//! 子模块：download/filename/cleanup/memory/mod_tools/crash_analyzer/
//! screenshot/resourcepack/version_json/archive/network/nbt/picker_window。
//! 种子地图工具已迁移至前端 WASM（cubiomes 编译为 WebAssembly，前端 Worker 直接调用）。

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;

pub mod archive;
pub mod cleanup;
pub mod crash_analyzer;
mod dispatcher;
pub mod download;
pub mod filename;
pub mod memory;
pub mod mod_tools;
pub mod nbt;
pub mod network;
pub mod picker_window;
pub mod resourcepack;
pub mod screenshot;
pub mod types;
pub mod version_json;

/// 统一工具 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `dispatcher::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn tools_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    dispatcher::dispatch(state, app, req).await
}
