//! 通用图片缓存组件
//!
//! 用于缓存皮肤、披风、头像等远程 PNG 图片。
//!
//! ## 设计（方案 C：混合缓存 + 自定义 URI scheme）
//!
//! - 首次加载：返回远程 URL，前端立即渲染；后端 `tokio::spawn` 异步下载到本地缓存
//! - 二次加载：返回自定义 URI scheme URL（`cache-image://{hash}.png`），零网络请求
//! - 缓存 key：URL 的 SHA1 hash，URL 变化时自动失效
//! - 下载完成：emit `image-cached` 事件通知前端刷新
//!
//! ## 安全性
//!
//! 不使用 Tauri 的 asset protocol（会暴露完整本地文件路径），
//! 而是注册自定义 URI scheme `cache-image`，前端只能通过 hash 请求，
//! 后端验证 hash 合法性后返回文件内容，无法构造任意路径读取本地文件。
//!
//! ## 缓存目录结构
//!
//! ```text
//! .Molaunch/cache/
//!   images/
//!     {sha1(url)}.png
//! ```
//!
//! ## 事件格式
//!
//! 事件名：`image-cached`
//! Payload：`{ remote_url: String, local_url: String }`

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use sha1::{Digest, Sha1};
use tauri::{AppHandle, Builder, Emitter, Runtime};
use tokio::sync::Mutex;

use crate::http::get_client;
use crate::storage::cache::Cache;

/// 缓存子目录名（相对于 cache 根目录）
const IMAGE_CACHE_DIR: &str = "images";
/// 自定义 URI scheme 名称
pub const CACHE_IMAGE_SCHEME: &str = "cache-image";

/// 正在下载中的 URL 集合（避免重复下载）
static IN_FLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn in_flight() -> &'static Mutex<HashSet<String>> {
    IN_FLIGHT.get_or_init(|| Mutex::new(HashSet::new()))
}

/// 缓存结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct CachedImage {
    /// 立即用于渲染的 URL
    pub url: String,
    /// 是否为本地缓存 URL（true 表示已缓存，无需网络）
    pub cached: bool,
}

/// 计算 URL 的 SHA1 hash（作为缓存文件名）
fn url_hash(url: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(url.as_bytes());
    hex::encode(hasher.finalize())
}

/// 缓存文件相对路径（相对于 cache 根目录）
fn cache_rel_path(url: &str) -> String {
    format!("{}/{}.png", IMAGE_CACHE_DIR, url_hash(url))
}

/// 缓存文件绝对路径
pub fn cache_abs_path(url: &str) -> PathBuf {
    Cache::instance().path(&cache_rel_path(url))
}

/// 生成自定义 URI scheme URL
///
/// Tauri 2 在 Windows/Android 上将自定义 scheme 映射为 `https://{scheme}.localhost`，
/// 在 macOS/Linux 上映射为 `{scheme}://localhost`。
/// 使用 HTTPS 格式确保 Chromium 允许跨源请求（自定义 scheme 原始格式会被 CORS 拦截）。
fn cache_image_url(url: &str) -> String {
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

/// 从自定义 URI scheme 请求路径中提取 hash
///
/// 支持两种 URL 格式：
/// 判断 URL 是否为 Tauri WebView 内部虚拟 URL（cache-image scheme）
///
/// Windows/Android 格式：`https://cache-image.localhost/{hash}.png`
/// macOS/Linux 格式：`cache-image://localhost/{hash}.png`
///
/// 这些 URL 只在 WebView 内部有效，后端 reqwest 等 HTTP 客户端无法访问，
/// 需要通过 `read_cache_by_url` 直接读取本地缓存文件。
pub fn is_cache_url(url: &str) -> bool {
    url.starts_with("https://cache-image.localhost/")
        || url.starts_with("cache-image://localhost/")
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
    let hash = filename.strip_suffix(".png").or_else(|| filename.strip_suffix(".PNG"))?;

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
    let path = Cache::instance().path(&rel);
    if path.exists() && path.is_file() {
        Some(path)
    } else {
        None
    }
}

/// 获取图片 URL（缓存优先，未命中时异步预下载）
///
/// - 如果本地缓存存在，返回自定义 URI scheme URL（`cached: true`）
/// - 如果本地缓存不存在，返回远程 URL（`cached: false`），并 spawn 异步任务下载
///
/// # 参数
/// - `remote_url`: 远程图片 URL（不可是 cache-image 虚拟 URL）
/// - `app`: Tauri AppHandle，用于下载完成后 emit 事件
///
/// # 防御性检查
/// 如果传入的 URL 已经是 cache-image 虚拟 URL（误用），
/// 直接返回它本身标记为 cached，避免用 reqwest 去下载虚拟 URL 导致连接失败。
pub async fn get_image_url(remote_url: &str, app: Option<AppHandle>) -> CachedImage {
    // 防御：如果误传 cache-image 虚拟 URL，直接返回，不发起 reqwest 下载
    if is_cache_url(remote_url) {
        return CachedImage {
            url: remote_url.to_string(),
            cached: true,
        };
    }

    let rel = cache_rel_path(remote_url);

    // 缓存命中：返回自定义 URI scheme URL
    if Cache::instance().exists(&rel) {
        return CachedImage {
            url: cache_image_url(remote_url),
            cached: true,
        };
    }

    // 缓存未命中：返回远程 URL，异步下载
    let url = remote_url.to_string();
    if let Some(app) = app {
        spawn_download(url.clone(), app);
    }

    CachedImage {
        url: remote_url.to_string(),
        cached: false,
    }
}

/// 异步下载图片到缓存（带去重）
fn spawn_download(remote_url: String, app: AppHandle) {
    // 检查是否已在下载
    let url_for_check = remote_url.clone();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        // 去重检查
        {
            let mut set = in_flight().lock().await;
            if set.contains(&url_for_check) {
                return;
            }
            set.insert(url_for_check.clone());
        }
        let url_for_remove = url_for_check.clone();

        // 执行下载
        let result = download_image(&remote_url).await;

        // 移除 in-flight 标记
        {
            let mut set = in_flight().lock().await;
            set.remove(&url_for_remove);
        }

        // 下载成功后 emit 事件
        if let Ok(()) = result {
            let local_url = cache_image_url(&remote_url);
            let payload = serde_json::json!({
                "remote_url": remote_url,
                "local_url": local_url,
            });
            if let Err(e) = app_clone.emit("image-cached", payload) {
                crate::log_warn!("[ImageCache] emit image-cached failed: {}", e);
            }
        }
    });
}

/// 下载图片并写入缓存
async fn download_image(remote_url: &str) -> anyhow::Result<()> {
    let client = get_client();
    let response = client
        .get(remote_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("download image failed: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "download image HTTP {}: {}",
            response.status(),
            remote_url
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("read image bytes failed: {}", e))?;

    let rel = cache_rel_path(remote_url);
    Cache::instance().write_bytes(&rel, &bytes)?;

    crate::log_info!(
        "[ImageCache] 已缓存: {} ({} 字节)",
        remote_url,
        bytes.len()
    );

    Ok(())
}

/// 清除指定 URL 的缓存（用于强制刷新）
pub fn invalidate(remote_url: &str) -> anyhow::Result<()> {
    let rel = cache_rel_path(remote_url);
    Cache::instance().remove(&rel)
}

/// 清空所有图片缓存
pub fn clear_all() -> anyhow::Result<()> {
    Cache::instance().clear_dir(IMAGE_CACHE_DIR)
}

// ============================================================================
// Tauri URI scheme 注册
// ============================================================================

/// 在 Tauri Builder 上注册 `cache-image` 自定义 URI scheme 协议
///
/// 将图片缓存协议处理从 `lib.rs` 抽离到此模块，`lib.rs` 只需调用
/// `image_cache::register_uri_scheme(builder)` 即可完成注册。
///
/// ## 协议行为
///
/// - 请求格式：`https://cache-image.localhost/{hash}.png`（Windows/Android）
///   或 `cache-image://localhost/{hash}.png`（macOS/Linux）
/// - hash 必须为 40 位十六进制（SHA1），否则返回 403
/// - 仅在 `images/` 子目录下查找文件，防止路径遍历攻击
/// - 所有响应附带 `Access-Control-Allow-Origin: *` 头
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
