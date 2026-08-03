//! 系统模块分发层（system_manager 的 dispatch 转发 + 域内共享配置更新 helper）

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::AppHandle;

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
/// `super::manager::dispatch` 分发。注册 20 个 action，分组：
/// game_dir(7) / config(2) / developer(6) / about(1) / logger(3) / updater(2)。
pub(crate) async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    super::manager::dispatch(state, app, req).await
}
