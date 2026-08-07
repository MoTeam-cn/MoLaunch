//! 实验性功能命令模块：提供开关控制下的聊天、日志分析与 Agent 命令入口。

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;

pub mod agent;
pub mod db;
pub mod manager;
pub mod types;

/// 实验性功能命令统一入口
#[tauri::command]
pub async fn experimental_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
