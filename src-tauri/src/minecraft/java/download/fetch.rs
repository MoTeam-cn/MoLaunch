//! HTTP 文本获取与索引/清单拉取
//!
//! `fetch_text_with_fallback` 低层多源回退；`fetch_index` 拉取 all.json；`fetch_manifest` 拉取 manifest.json

use crate::minecraft::sources::{build_replace_urls, DownloadSourceMode};

use super::constants::{DOWNLOAD_DOMAIN_REPLACEMENTS, JAVA_RUNTIME_INDEX_OFFICIAL};
use super::types::RuntimeManifest;

/// 带回退的文本获取（依次尝试 URL 列表）
pub async fn fetch_text_with_fallback(
    client: &reqwest::Client,
    urls: &[String],
) -> Result<String, String> {
    let mut last_err = String::new();
    for url in urls {
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(text) => return Ok(text),
                Err(e) => last_err = format!("读取失败: {}", e),
            },
            Ok(resp) => last_err = format!("HTTP {}", resp.status()),
            Err(e) => last_err = format!("请求失败: {}", e),
        }
    }
    Err(format!("所有源均失败: {}", last_err))
}

/// 阶段 1：拉取并解析 Mojang all.json 索引
pub async fn fetch_index(
    client: &reqwest::Client,
    mirror_url: Option<&str>,
    mode: DownloadSourceMode,
) -> Result<serde_json::Value, String> {
    let urls = build_replace_urls(
        JAVA_RUNTIME_INDEX_OFFICIAL,
        mirror_url,
        DOWNLOAD_DOMAIN_REPLACEMENTS,
        mode,
    );
    let text = fetch_text_with_fallback(client, &urls).await?;
    serde_json::from_str(&text).map_err(|e| format!("解析 Java 索引失败: {}", e))
}

/// 阶段 3：拉取并解析 manifest.json 文件清单
pub async fn fetch_manifest(
    client: &reqwest::Client,
    manifest_url: &str,
    mirror_url: Option<&str>,
    mode: DownloadSourceMode,
) -> Result<RuntimeManifest, String> {
    let urls = build_replace_urls(manifest_url, mirror_url, DOWNLOAD_DOMAIN_REPLACEMENTS, mode);
    let text = fetch_text_with_fallback(client, &urls).await?;
    serde_json::from_str(&text).map_err(|e| format!("解析文件清单失败: {}", e))
}
