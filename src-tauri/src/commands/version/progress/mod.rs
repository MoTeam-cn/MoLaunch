//! 统一下载进度 IPC 入口
//!
//! 全部进度逻辑（get_download_progress / 暂停 / 恢复 / 取消）位于 `manager`。
//! 注：本文件保留 `version_progress_manager` 命令转发函数——`#[tauri::command]`
//! 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出。

mod manager;

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// 统一下载进度 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn version_progress_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
