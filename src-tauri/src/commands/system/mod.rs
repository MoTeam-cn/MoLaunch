//! 系统命令模块
//! 子模块：about / apply_config / config / config_manager / developer / game_dir /
//! manager / updater。game、proxy 模块保留为占位（后续如需专属命令可扩展）。

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;

pub mod about;
pub mod apply_config;
pub mod config;
pub mod config_manager;
pub mod developer;
mod dispatcher;
mod game;
pub mod game_dir;
pub(crate) mod manager;
pub mod memory_push;
mod proxy;
pub mod updater;

pub use about::*;
pub use apply_config::*;
pub use config::*;
pub use developer::*;
pub use game_dir::*;

pub(crate) use dispatcher::update_config;

/// 统一系统模块 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `dispatcher::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn system_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    dispatcher::dispatch(state, app, req).await
}
