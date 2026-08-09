//! 程序重启快照命令（SDK 加密保存 / 解密读取，供提权重启等重启流程使用）

mod manager;

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// 统一重启快照 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到 `manager::dispatch` 分发：
/// - `encrypt`：`params.data`（快照 JSON 明文）→ SDK AES 密文
/// - `decrypt`：`params.data`（快照密文）→ SDK 解密明文
#[tauri::command]
pub async fn relaunch_snapshot(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
