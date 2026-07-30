//! 系统命令模块

pub mod about;
pub mod apply_config;
pub mod config;
pub mod developer;
mod game;
pub mod game_dir;
mod proxy;
pub mod updater;

pub use about::*;
pub use apply_config::*;
pub use config::*;
pub use developer::*;
pub use game_dir::*;

// game/proxy 模块保留为占位（后续如需专属命令可扩展），
// 所有 get/set 由 `get_config` / `apply_config` 统一处理。

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// 更新配置并保存
pub(crate) async fn update_config<F>(state: &AppState, updater: F) -> Result<(), String>
where
    F: FnOnce(&mut crate::state::AppConfig),
{
    let mut config = state.config.lock().await;
    updater(&mut config);
    let config_clone = config.clone();
    drop(config);

    crate::config::save_config(&config_clone)?;
    Ok(())
}


/// 统一系统模块 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::system_manager::dispatch` 进行 action 分发。
///
/// 注册的 action（20 个）：
/// - game_dir（7 个）：`open_game_dir` / `open_path` / `reveal_in_explorer`
///   / `get_game_dir` / `write_text_file` / `get_system_memory` / `set_game_dir`
/// - config（2 个）：`get_config_path` / `save_config_to_file`
/// - developer（6 个）：`is_developer_unlocked` / `unlock_developer_mode`
///   / `lock_developer_mode` / `get_storage_dirs` / `get_system_info` / `get_cache_stats`
/// - about（1 个）：`get_about_data`
/// - logger（3 个）：`get_log_path` / `list_log_files` / `read_log_file`
/// - updater（2 个）：`check_update` / `download_and_install_update`
#[tauri::command]
pub async fn system_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::system_manager::dispatch(state, app, req).await
}
