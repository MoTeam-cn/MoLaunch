//! 版本下载进度统一分发逻辑（version_progress_manager 的命令层实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，6 个 action 覆盖
//! 进度查询 / 暂停 / 恢复 / 取消。除 `get_download_progress` 外均操作原子标志位。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

use super::{
    cancel_download, get_download_progress, is_downloading, pause_download,
    reset_download_progress, resume_download,
};

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    d.register(
        "get_download_progress",
        handler!(state, _app, _params, {
            let r = get_download_progress(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "is_downloading",
        handler!(state, _app, _params, {
            let r = is_downloading(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "reset_download_progress",
        handler!(state, _app, _params, {
            reset_download_progress(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "cancel_download",
        handler!(state, _app, _params, {
            cancel_download(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "pause_download",
        handler!(state, _app, _params, {
            pause_download(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "resume_download",
        handler!(state, _app, _params, {
            resume_download(&state).await?;
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
