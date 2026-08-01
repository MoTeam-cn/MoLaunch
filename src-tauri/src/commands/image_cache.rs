//! 通用图片缓存命令
//! 提供通用的图片缓存接口，前端通过 `image_cache_manager` IPC 入口
//! 将任意远程图片 URL 转为缓存 URL（方案 C：混合缓存）。
//! 适用于皮肤、披风、头像、缩略图等所有需要缓存的远程图片场景。

use crate::error_util::log_err;
use crate::minecraft::image_cache::{self, CachedImage};
use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// 统一图片缓存 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::image_cache_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn image_cache_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::image_cache_manager::dispatch(state, app, req).await
}

/// 获取图片的缓存 URL（通用接口）
///
/// - 缓存命中：返回 `cache-image://{hash}.png` 本地 URL（`cached: true`）
/// - 缓存未命中：返回原始远程 URL（`cached: false`），后端异步下载到缓存，完成后 emit `image-cached` 事件
pub async fn get_cached_image_url(url: String, app: &AppHandle) -> Result<CachedImage, String> {
    Ok(image_cache::get_image_url(&url, Some(app.clone())).await)
}

/// 失效指定 URL 的图片缓存（强制刷新）
pub fn invalidate_cached_image(url: String) -> Result<(), String> {
    image_cache::invalidate(&url).map_err(log_err("Failed to invalidate cached image"))
}

/// 清空所有图片缓存
pub fn clear_image_cache() -> Result<(), String> {
    image_cache::clear_all().map_err(log_err("Failed to clear image cache"))
}
