//! 配置文件相关命令
//!
//! 子模块函数接收 `&AppState`，由 `utils::config_manager::dispatch` /
//! `utils::system_manager::dispatch` 反序列化参数后调用。

use crate::log_info;
use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// 获取配置文件路径
pub async fn get_config_path() -> Result<String, String> {
    let storage = crate::storage::Storage::instance();
    Ok(storage.config_path().to_string_lossy().to_string())
}

/// 手动保存配置到文件
pub async fn save_config_to_file(state: &AppState) -> Result<(), String> {
    let config = state.config.lock().await;
    crate::config::save_config(&config)?;
    log_info!("Config saved manually");
    Ok(())
}

/// 统一配置管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::config_manager::dispatch` 进行 action 分发。
///
/// 注册的 action（2 个）：
/// - `get_config`：读取配置（扁平化数组）
/// - `apply_config`：统一配置更新
#[tauri::command]
pub async fn config_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::config_manager::dispatch(state, app, req).await
}
