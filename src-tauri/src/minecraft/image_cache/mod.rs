//! 通用图片缓存组件（皮肤/披风/头像等远程 PNG）
//!
//! 混合缓存：首次返回远程 URL，后端异步下载到本地；二次返回自定义 URI scheme；
//! 子模块：store / download / cleanup。

mod cleanup;
mod download;
mod store;

use tauri::{Builder, Runtime};

pub use cleanup::{clear_all, invalidate};
pub use download::get_image_url;
pub use store::{
    cache_abs_path, cache_path_by_url, find_cache_by_hash, is_cache_url, parse_hash_from_request,
    read_cache_by_url, CachedImage, CACHE_IMAGE_SCHEME,
};

// Tauri URI scheme 注册

/// 在 Tauri Builder 上注册 `cache-image` 自定义 URI scheme 协议
///
/// `lib.rs` 调 `image_cache::register_uri_scheme(builder)` 完成注册。协议行为：
/// - 请求格式 `https://cache-image.localhost/{hash}.png`（Win/Android）或
///   `cache-image://localhost/{hash}.png`（macOS/Linux）
/// - hash 必须 40 位十六进制（SHA1），否则 403；仅在 `images/` 子目录查找，防路径遍历
/// - 响应附带 `Access-Control-Allow-Origin: *`
pub fn register_uri_scheme<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.register_uri_scheme_protocol(CACHE_IMAGE_SCHEME, |_ctx, request| {
        handle_cache_image_request(&request)
    })
}

/// 处理 `cache-image` 协议请求，返回响应
fn handle_cache_image_request(
    request: &tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    // 从请求 URI 中提取 hash
    let uri = request.uri().to_string();

    // 解析 hash
    let hash = match parse_hash_from_request(&uri) {
        Some(h) => h,
        None => {
            crate::log_warn!("[ImageCache] 无效的缓存图片请求: {}", uri);
            return empty_response(403);
        }
    };

    // 根据 hash 查找缓存文件
    match find_cache_by_hash(&hash) {
        Some(path) => match std::fs::read(&path) {
            Ok(bytes) => tauri::http::Response::builder()
                .status(200)
                .header("Content-Type", "image/png")
                .header("Cache-Control", "public, max-age=86400")
                .header("Access-Control-Allow-Origin", "*")
                .body::<Vec<u8>>(bytes)
                .unwrap(),
            Err(e) => {
                crate::log_warn!("[ImageCache] 读取缓存文件失败: {}", e);
                empty_response(500)
            }
        },
        None => {
            crate::log_warn!("[ImageCache] 缓存文件不存在: {}", hash);
            empty_response(404)
        }
    }
}

/// 构造空的错误响应（附带 CORS 头）
fn empty_response(status: u16) -> tauri::http::Response<Vec<u8>> {
    tauri::http::Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .body::<Vec<u8>>(Vec::new())
        .unwrap()
}