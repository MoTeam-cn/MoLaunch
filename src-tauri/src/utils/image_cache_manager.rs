//! 通用图片缓存统一分发逻辑（image_cache_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 3 个 image_cache action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（3 个）：
//! - `get_cached_image_url`：获取图片缓存 URL（命中返回本地 URL，未命中返回远程 URL 并异步缓存）
//! - `invalidate_cached_image`：失效指定 URL 的缓存
//! - `clear_image_cache`：清空所有图片缓存
//!
//! 注意：image_cache 命令不需要 `AppState`，handler 内用 `_state` 忽略；
//! `get_cached_image_url` 需要 `AppHandle`（用于 emit `image-cached` 事件）。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::image_cache;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// action 参数
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UrlParams {
    url: String,
}

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("get_cached_image_url", handler!(_state, app, params, {
        let p: UrlParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = image_cache::get_cached_image_url(p.url, &app).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("invalidate_cached_image", handler!(_state, _app, params, {
        let p: UrlParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        image_cache::invalidate_cached_image(p.url)?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("clear_image_cache", handler!(_state, _app, _params, {
        image_cache::clear_image_cache()?;
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
