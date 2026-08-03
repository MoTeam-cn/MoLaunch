//! 图片缓存存储
//!
//! 路径/hash 计算、自定义 URI scheme 解析与本地文件读取。

use crate::utils::cache;
use sha1::{Digest, Sha1};
use std::path::PathBuf;

/// 缓存子目录名（相对于 cache 根目录）
pub(super) const IMAGE_CACHE_DIR: &str = "images";
/// 自定义 URI scheme 名称
pub const CACHE_IMAGE_SCHEME: &str = "cache-image";

/// 缓存结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct CachedImage {
    /// 立即用于渲染的 URL
    pub url: String,
    /// 是否为本地缓存 URL（true 表示已缓存，无需网络）
    pub cached: bool,
}

/// 计算 URL 的 SHA1 hash（作为缓存文件名）
pub(super) fn url_hash(url: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(url.as_bytes());
    hex::encode(hasher.finalize())
}

/// 缓存文件相对路径（相对于 cache 根目录）
pub(super) fn cache_rel_path(url: &str) -> String {
    format!("{}/{}.png", IMAGE_CACHE_DIR, url_hash(url))
}

/// 缓存文件绝对路径
pub fn cache_abs_path(url: &str) -> PathBuf {
    cache::path(&cache_rel_path(url))
}

/// 生成自定义 URI scheme URL
///
/// Tauri 2 在 Windows/Android 上将自定义 scheme 映射为 `https://{scheme}.localhost`，
/// 在 macOS/Linux 上映射为 `{scheme}://localhost`。
/// 使用 HTTPS 格式确保 Chromium 允许跨源请求（自定义 scheme 原始格式会被 CORS 拦截）。
///
/// 平台 URL 格式差异（由 Tauri WebView 内核决定）：
/// - Windows (WebView2)：`https://cache-image.localhost/{hash}.png`
/// - macOS / Linux (WebKitGTK)：`cache-image://localhost/{hash}.png`
///
/// 注：`cfg(not(target_os = "windows"))` 理论上含 Android，但项目不支持 Android
/// 构建，实际仅覆盖 macOS / Linux。若未来支持 Android，需验证其 WebView
/// (Chromium 内核) 应走 `https://` 格式，届时改为显式 `cfg(any(target_os = "macos", target_os = "linux"))`。
pub(super) fn cache_image_url(url: &str) -> String {
    let hash = url_hash(url);
    #[cfg(target_os = "windows")]
    {
        format!("https://{}.localhost/{}.png", CACHE_IMAGE_SCHEME, hash)
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("{}://localhost/{}.png", CACHE_IMAGE_SCHEME, hash)
    }
}

/// 判断 URL 是否为 Tauri WebView 内部虚拟 URL（cache-image scheme）
///
/// Windows/Android 格式：`https://cache-image.localhost/{hash}.png`
/// macOS/Linux 格式：`cache-image://localhost/{hash}.png`
///
/// 这些 URL 只在 WebView 内部有效，后端 reqwest 等 HTTP 客户端无法访问，
/// 需要通过 `read_cache_by_url` 直接读取本地缓存文件。
pub fn is_cache_url(url: &str) -> bool {
    url.starts_with("https://cache-image.localhost/") || url.starts_with("cache-image://localhost/")
}

/// 从 cache-image 虚拟 URL 读取本地缓存文件内容
///
/// 如果 URL 不是 cache-image 虚拟格式或缓存文件不存在，返回 None。
/// 调用方应先判断返回值，None 时可回退到普通 HTTP 下载。
pub fn read_cache_by_url(url: &str) -> Option<Vec<u8>> {
    if !is_cache_url(url) {
        return None;
    }
    let hash = parse_hash_from_request(url)?;
    let cache_path = find_cache_by_hash(&hash)?;
    std::fs::read(&cache_path).ok()
}

/// 返回缓存文件的路径（如果 URL 是 cache-image 虚拟格式且缓存文件存在）
pub fn cache_path_by_url(url: &str) -> Option<PathBuf> {
    if !is_cache_url(url) {
        return None;
    }
    let hash = parse_hash_from_request(url)?;
    find_cache_by_hash(&hash)
}

/// - Windows/Android: `https://cache-image.localhost/{hash}.png`
/// - macOS/Linux: `cache-image://localhost/{hash}.png`
///
/// 返回：hash 值（用于查找缓存文件），格式不合法返回 None
pub fn parse_hash_from_request(uri: &str) -> Option<String> {
    // URI 格式：https://cache-image.localhost/{hash}.png
    // 提取路径部分（去掉 scheme 和 host），取最后一段作为 hash
    let path = uri.split('?').next()?; // 去掉 query string
    let path = path.split('#').next()?; // 去掉 fragment

    // 取路径最后一部分
    let filename = path.rsplit('/').next()?;
    // 去掉 .png 后缀
    let hash = filename
        .strip_suffix(".png")
        .or_else(|| filename.strip_suffix(".PNG"))?;

    // 验证 hash 格式（SHA1 是 40 位十六进制）
    if hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hash.to_string())
    } else {
        None
    }
}

/// 根据 hash 查找缓存文件路径
///
/// 用于 URI scheme handler：验证 hash 合法性后返回文件路径
pub fn find_cache_by_hash(hash: &str) -> Option<PathBuf> {
    let rel = format!("{}/{}.png", IMAGE_CACHE_DIR, hash);
    let path = cache::path(&rel);
    if path.exists() && path.is_file() {
        Some(path)
    } else {
        None
    }
}

// Tauri URI scheme 注册

/// 在 Tauri Builder 上注册 `cache-image` 自定义 URI scheme 协议
///
/// `lib.rs` 调 `image_cache::register_uri_scheme(builder)` 完成注册。协议行为：
/// - 请求格式 `https://cache-image.localhost/{hash}.png`（Win/Android）或
///   `cache-image://localhost/{hash}.png`（macOS/Linux）
/// - hash 必须 40 位十六进制（SHA1），否则 403；仅在 `images/` 子目录查找，防路径遍历
/// - 响应附带 `Access-Control-Allow-Origin: *`
pub fn register_uri_scheme<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
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
