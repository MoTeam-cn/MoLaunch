//! 图片下载与去重
//!
//! 未命中缓存时异步下载并写入存储，完成后 emit `image-cached` 通知前端。

use super::store::{cache_image_url, cache_rel_path, is_cache_url, CachedImage};
use crate::http::get_client;
use crate::utils::cache;
use std::collections::HashSet;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// 正在下载中的 URL 集合（避免重复下载）
static IN_FLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn in_flight() -> &'static Mutex<HashSet<String>> {
    IN_FLIGHT.get_or_init(|| Mutex::new(HashSet::new()))
}

/// 获取图片 URL（缓存优先，未命中时异步预下载）
///
/// - 缓存命中：返回自定义 URI scheme URL（`cached: true`）
/// - 未命中：返回远程 URL（`cached: false`），spawn 异步下载，完成后 emit `image-cached`
/// - `remote_url`：远程图片 URL（不可是 cache-image 虚拟 URL）；`app`：emit 事件用
/// - 防御：误传 cache-image 虚拟 URL 时直接返回 cached，避免 reqwest 下载虚拟 URL 失败
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
    if cache::exists(&rel) {
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
    cache::write_bytes(&rel, &bytes)?;

    crate::log_info!("[ImageCache] 已缓存: {} ({} 字节)", remote_url, bytes.len());

    Ok(())
}