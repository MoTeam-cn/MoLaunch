//! SDK 模块统一分发逻辑（sdk 域 manager 模块）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，4 个 action 均无参数：
//! `get_platform_info`（不需要 state）/ `get_sdk_version` / `is_sdk_initialized` /
//! `get_device_id`（后 3 个需 `&state` 访问 `state.sdk` 锁）。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use super::*;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "get_platform_info",
        handler!(_state, _app, _params, {
            let r = get_platform_info().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_sdk_version",
        handler!(state, _app, _params, {
            let r = get_sdk_version(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "is_sdk_initialized",
        handler!(state, _app, _params, {
            let r = is_sdk_initialized(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_device_id",
        handler!(state, _app, _params, {
            let r = get_device_id(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

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
