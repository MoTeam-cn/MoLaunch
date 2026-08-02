//! 系统命令模块

pub mod about;
pub mod apply_config;
pub mod config;
pub mod config_manager;
pub mod developer;
mod game;
pub mod game_dir;
pub(crate) mod manager;
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
/// 接收 `ActionRequest { action, params }`，转发到
/// `manager::dispatch` 分发。注册 20 个 action，分组：
/// game_dir(7) / config(2) / developer(6) / about(1) / logger(3) / updater(2)。
/// 具体 action 名见 `manager::dispatch` 注册表。
#[tauri::command]
pub async fn system_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
