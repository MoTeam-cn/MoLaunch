//! 版本启动命令
//! 子模块：manager（Dispatcher 分发）、build_config / build / spawn / failure、
//! types（共享类型 GameExitEvent）。

mod build;
mod build_config;
mod failure;
mod manager;
mod preview;
mod spawn;
mod types;

pub use build::*;
pub use preview::*;
pub use spawn::*;
pub use types::GameExitEvent;

/// 版本启动管理统一 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 原 7 个独立 Tauri 命令（6 个 launch + 1 个 script_export）均通过此入口聚合调用。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn version_launch_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
