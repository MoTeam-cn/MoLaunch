//! 自定义布局 URL 加载 + 本地缓存
//!
//! - `load_custom_layout`：加载 URL 自定义布局内容，支持本地缓存
//!
//! 缓存策略：
//! - URL 的 sha256 哈希作为缓存文件名（`<sha256>.txt`）
//! - 缓存目录：`.Molaunch/cache/custom_layout/`
//! - TTL：24 小时（基于文件修改时间）
//! - `force_refresh=true` 时强制忽略本地缓存重新下载
//!
//! URL 协议校验：仅允许 http/https，拒绝 file://、data: 等。

use crate::error_util::log_err;
use crate::{http, log_info, log_warn, utils::cache};
use sha2::{Digest, Sha256};

/// 缓存 TTL（24 小时）
const TTL_SECONDS: u64 = 24 * 60 * 60;

/// 加载 URL 自定义布局内容
///
/// 流程：URL 协议校验 → 计算缓存文件名 → 检查本地缓存（命中且未过期 → 返回）
/// → 下载 URL 内容 → 写入缓存 → 返回内容
#[tauri::command]
pub async fn load_custom_layout(
    url: String,
    force_refresh: Option<bool>,
) -> Result<String, String> {
    // 1. URL 协议校验
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("URL must be http/https: {}", url));
    }

    // 2. 计算缓存文件名（sha256 哈希）
    let cache_filename = format!("{}.txt", hash_url(&url));
    let cache_rel = format!("custom_layout/{}", cache_filename);

    // 3. 检查本地缓存
    let force = force_refresh.unwrap_or(false);

    if !force && cache::exists(&cache_rel) {
        // 检查 TTL（基于文件修改时间）
        let cache_path = cache::path(&cache_rel);
        if let Ok(metadata) = std::fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() < TTL_SECONDS {
                        // 命中缓存
                        log_info!("自定义布局命中缓存: {}", url);
                        return cache::read(&cache_rel).map_err(log_err("Failed to read layout cache"));
                    }
                    log_info!("自定义布局缓存已过期: {}", url);
                }
            }
        }
    }

    // 4. 下载 URL 内容（使用全局 HTTP 客户端，自带 UA 标识）
    let content = http::fetch_url(&url)
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    // 5. 写入缓存（失败仅告警，不阻塞返回内容）
    if let Err(e) = cache::write(&cache_rel, &content) {
        log_warn!("写入自定义布局缓存失败: {}", e);
    }

    log_info!(
        "自定义布局已从 URL 加载: {} ({} 字节)",
        url,
        content.len()
    );

    Ok(content)
}

/// 计算 URL 的 sha256 十六进制哈希
fn hash_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}
