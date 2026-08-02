//! 通用图片缓存统一分发逻辑（image_cache 域 manager 模块）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，3 个 action：
//! `get_cached_image_url` / `invalidate_cached_image` / `clear_image_cache`。
//! 不需要 `AppState`；`get_cached_image_url` 需要 `AppHandle`（emit `image-cached`）。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use super::*;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UrlParams {
    url: String,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "get_cached_image_url",
        handler!(_state, app, params, {
            let p: UrlParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = get_cached_image_url(p.url, &app).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "invalidate_cached_image",
        handler!(_state, _app, params, {
            let p: UrlParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            invalidate_cached_image(p.url)?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "clear_image_cache",
        handler!(_state, _app, _params, {
            clear_image_cache()?;
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
