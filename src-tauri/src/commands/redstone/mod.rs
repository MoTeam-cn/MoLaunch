//! 红石联机命令模块（hongshi 内核）
//! 提供红石联机单一 IPC 入口 redstone_manager，action 由 manager::dispatch 分发
//! （redstone_get_servers / redstone_start / redstone_status / redstone_stop
//!  / redstone_log_files / redstone_read_log）。

pub(crate) mod log;
pub(crate) mod manager;
pub(crate) mod tunnel;

/// 统一红石联机 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到 `manager::dispatch` 分发。
#[tauri::command]
pub async fn redstone_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
