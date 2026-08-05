//! AI 模块（统一 IPC 入口）
//! 子模块：manager（action 分发）、types（IPC 类型）。
//! 服务层逻辑位于 `crate::ai_core`（不含 Tauri 依赖），
//! 服务为本地 OpenAI 兼容 API（如 Ollama / LM Studio）。

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;

pub mod manager;
pub mod types;

/// 统一 AI IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn ai_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
