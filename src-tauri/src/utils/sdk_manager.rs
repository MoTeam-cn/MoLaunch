//! SDK 模块统一分发逻辑（sdk_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 5 个 sdk action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（5 个）：
//! - `get_platform_info`：获取平台信息（不需要 state）
//! - `get_sdk_version`：获取 SDK 版本
//! - `is_sdk_initialized`：检查 SDK 是否已初始化
//! - `get_device_id`：获取设备 ID
//! - `check_update_lite`：检查更新（轻量版）
//!
//! 注意：5 个 action 均无参数，handler 内用 `_params` 忽略；
//! `get_platform_info` 不需要 state，handler 内用 `_state` / `_app` 忽略。
//! 其余 4 个 action 需要 `&state` 访问 `state.sdk` 锁。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::commands::sdk;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("get_platform_info", handler!(_state, _app, _params, {
        let r = sdk::get_platform_info().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_sdk_version", handler!(state, _app, _params, {
        let r = sdk::get_sdk_version(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("is_sdk_initialized", handler!(state, _app, _params, {
        let r = sdk::is_sdk_initialized(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_device_id", handler!(state, _app, _params, {
        let r = sdk::get_device_id(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("check_update_lite", handler!(state, _app, _params, {
        let r = sdk::check_update_lite(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d
});

/// 分发入口
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}
