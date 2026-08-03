//! 认证模块统一分发逻辑（auth 域 meta_manager 模块）
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，28 个 auth action 在
//! `once_cell::sync::Lazy` 初始化时按域分组注册：offline / microsoft / authlib
//! 及会话通用操作。`ms_login_web_start` 等需要 `&app` 的 action 在对应分组内处理。

mod authlib;
mod microsoft;
mod offline;

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    offline::register(&mut d);
    microsoft::register(&mut d);
    authlib::register(&mut d);

    d.register(
        "get_login_status",
        handler!(state, _app, _params, {
            let r = crate::commands::auth::account::session::get_login_status(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "logout",
        handler!(state, _app, _params, {
            crate::commands::auth::account::session::logout(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d
});

/// 按 `req.action` 分发到对应 handler
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}