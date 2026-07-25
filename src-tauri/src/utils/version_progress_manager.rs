//! 下载进度统一分发逻辑（version_progress_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 6 个 download progress action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（6 个，均无额外参数，仅需 state）：
//! - `get_download_progress`：获取下载进度快照
//! - `is_downloading`：检查是否正在下载
//! - `reset_download_progress`：重置下载进度
//! - `cancel_download`：取消下载（设置 cancel_flag）
//! - `pause_download`：暂停下载（设置 pause_flag）
//! - `resume_download`：恢复下载（清除 pause_flag）
//!
//! 注意：所有 handler 都不需要 `AppHandle`，用 `_app` 忽略；params 为 null，用 `_params` 忽略。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::commands::version::progress;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("get_download_progress", handler!(state, _app, _params, {
        let r = progress::get_download_progress(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("is_downloading", handler!(state, _app, _params, {
        let r = progress::is_downloading(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("reset_download_progress", handler!(state, _app, _params, {
        progress::reset_download_progress(&state).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("cancel_download", handler!(state, _app, _params, {
        progress::cancel_download(&state).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("pause_download", handler!(state, _app, _params, {
        progress::pause_download(&state).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("resume_download", handler!(state, _app, _params, {
        progress::resume_download(&state).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
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
