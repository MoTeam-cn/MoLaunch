//! 下载工具函数：源模式获取、URL 构建、带重试抓取

use std::path::Path;

use crate::{http, log_debug};

use super::super::sources::{self, DownloadSourceMode};
use super::manager::DownloadManager;

/// 从 DownloadManager 获取 source_mode（用于构造 URL）
pub fn source_mode_of(manager: &DownloadManager) -> DownloadSourceMode {
    manager.source_mode()
}

/// 构建 launcher/meta URL 列表
pub fn build_launcher_meta_urls(
    original: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> Vec<String> {
    sources::build_replace_urls(
        original,
        mirror_url,
        sources::MOJANG_REPLACEMENTS,
        source_mode,
    )
}

/// 带重试的下载
pub async fn fetch_with_retry(
    primary_url: &str,
    local_path: &Path,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<String> {
    let urls = build_launcher_meta_urls(primary_url, mirror_url, source_mode);

    for url in &urls {
        match fetch_url_to_file(url, local_path).await {
            Ok(content) => return Ok(content),
            Err(e) => {
                log_debug!("Failed to fetch from {}: {}", url, e);
                continue;
            }
        }
    }

    Err(anyhow::anyhow!("All download sources failed"))
}

/// 下载 URL 内容到文件
async fn fetch_url_to_file(url: &str, local_path: &Path) -> anyhow::Result<String> {
    http::fetch_url_to_file(url, local_path).await
}

/// 获取 URL 内容
pub async fn fetch_url(url: &str) -> anyhow::Result<String> {
    http::fetch_url(url).await
}
