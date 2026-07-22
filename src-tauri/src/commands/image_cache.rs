//! 通用图片缓存命令
//!
//! 提供通用的图片缓存接口，前端可通过 `get_cached_image_url` 命令
//! 将任意远程图片 URL 转为缓存 URL（方案 C：混合缓存）。
//!
//! 适用于皮肤、披风、头像、缩略图等所有需要缓存的远程图片场景。

use crate::error_util::log_err;
use crate::minecraft::image_cache::{self, CachedImage};
use tauri::AppHandle;

/// 获取图片的缓存 URL（通用接口）
///
/// - 缓存命中：返回 `cache-image://{hash}.png` 本地 URL（`cached: true`）
/// - 缓存未命中：返回原始远程 URL（`cached: false`），后端异步下载到缓存，完成后 emit `image-cached` 事件
///
/// # 参数
/// - `url`: 远程图片 URL
///
/// # 返回
/// `CachedImage { url, cached }`
#[tauri::command]
pub async fn get_cached_image_url(url: String, app: AppHandle) -> Result<CachedImage, String> {
    Ok(image_cache::get_image_url(&url, Some(app)).await)
}

/// 失效指定 URL 的图片缓存（强制刷新）
#[tauri::command]
pub fn invalidate_cached_image(url: String) -> Result<(), String> {
    image_cache::invalidate(&url).map_err(log_err("Failed to invalidate cached image"))
}

/// 清空所有图片缓存
#[tauri::command]
pub fn clear_image_cache() -> Result<(), String> {
    image_cache::clear_all().map_err(log_err("Failed to clear image cache"))
}
