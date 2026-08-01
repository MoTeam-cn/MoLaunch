//! 下载进度统一分发逻辑（version_progress_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，6 个 action 均无额外参数仅需 state：
//! `get_download_progress` / `is_downloading` / `reset_download_progress` /
//! `cancel_download` / `pause_download` / `resume_download`。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::commands::version::progress;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "get_download_progress",
        handler!(state, _app, _params, {
            let r = progress::get_download_progress(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "is_downloading",
        handler!(state, _app, _params, {
            let r = progress::is_downloading(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "reset_download_progress",
        handler!(state, _app, _params, {
            progress::reset_download_progress(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "cancel_download",
        handler!(state, _app, _params, {
            progress::cancel_download(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "pause_download",
        handler!(state, _app, _params, {
            progress::pause_download(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "resume_download",
        handler!(state, _app, _params, {
            progress::resume_download(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
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
