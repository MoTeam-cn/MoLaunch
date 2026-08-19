//! GitHub 下载公共组件：release 版本查询 + 资产下载（镜像优先 + 官方保底）
//! 供 easytier 内核、frpc 等外部二进制按需下载复用（repo 参数化）。

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::utils::probe::{pick_fastest, probe_urls};

/// GitHub 镜像源（type: path 追加路径 / type: full 追加完整 URL）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubProxy {
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub base: String,
    #[serde(default)]
    pub name: String,
}

/// 官方下载源
const GITHUB_DOWNLOAD_BASE: &str = "https://github.com";
/// GitHub API 主源
const GITHUB_API_PRIMARY: &str = "https://api.github.com";
/// GitHub API 备选源（仅 API 功能）
const GITHUB_API_FALLBACK: &str = "https://github-api.mocdn.net";

/// 构造镜像源下载 URL（type: path 追加路径 / type: full 追加完整 GitHub URL）
pub fn build_proxy_url(proxy: &GithubProxy, repo: &str, version: &str, asset: &str) -> String {
    let base = proxy.base.trim_end_matches('/');
    if proxy.proxy_type == "path" {
        format!("{base}/{repo}/releases/download/v{version}/{asset}")
    } else {
        // full 模式：base 与完整 GitHub URL 之间补 `/`（base 可能无尾斜杠）
        format!("{base}/{GITHUB_DOWNLOAD_BASE}/{repo}/releases/download/v{version}/{asset}")
    }
}

/// 查询指定仓库最新版本号（主源失败回退备选；失败返回错误由前端提示）
pub async fn fetch_latest_release(client: &reqwest::Client, repo: &str) -> Result<String, String> {
    let primary = format!("{GITHUB_API_PRIMARY}/repos/{repo}/releases/latest");
    match fetch_tag_name(client, &primary).await {
        Ok(tag) => Ok(tag),
        Err(e) => {
            crate::log_warn!("[GitHub] API 主源失败: {e}，回退备选源");
            let fallback = format!("{GITHUB_API_FALLBACK}/repos/{repo}/releases/latest");
            fetch_tag_name(client, &fallback).await
        }
    }
}

/// 请求 release API 解析 tag_name（去 v 前缀，单请求 30s 超时）
async fn fetch_tag_name(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let value: serde_json::Value = resp.json().await.map_err(|e| format!("解析失败: {e}"))?;
    let tag = value
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "响应缺少 tag_name".to_string())?;
    Ok(tag.trim_start_matches('v').to_string())
}

/// 镜像源测速筛选：随机抽 `sample` 个，并发测速（禁止重定向），
/// 返回最快的前 `limit` 个（供前端启动时筛选默认源）。
pub async fn probe_fastest_proxies(
    proxies: &[GithubProxy],
    repo: &str,
    version: &str,
    asset: &str,
    sample: usize,
    limit: usize,
) -> Vec<GithubProxy> {
    use rand::seq::SliceRandom;
    let mut pool: Vec<&GithubProxy> = proxies.iter().collect();
    pool.shuffle(&mut rand::thread_rng());
    let sampled: Vec<&GithubProxy> = pool.into_iter().take(sample).collect();
    crate::log_debug!(
        "[GitHub] 测速抽样: 从 {} 个源中抽 {} 个",
        proxies.len(),
        sampled.len()
    );
    let urls: Vec<String> = sampled
        .iter()
        .map(|p| build_proxy_url(p, repo, version, asset))
        .collect();
    let Ok(results) = probe_urls(&urls, None).await else {
        return Vec::new();
    };
    let usable = results.len();
    let picked: Vec<GithubProxy> = results
        .into_iter()
        .take(limit)
        .filter_map(|(_, url)| {
            sampled
                .iter()
                .find(|p| build_proxy_url(p, repo, version, asset) == url)
                .map(|p| (*p).clone())
        })
        .collect();
    crate::log_debug!(
        "[GitHub] 测速筛选: 可用 {} 个，返回最快 {} 个",
        usable,
        picked.len()
    );
    picked
}

/// 流式下载文件（reqwest bytes_stream，单请求 120s 超时，按字节回调进度：done/total）
pub async fn download_to(
    client: &reqwest::Client,
    url: &str,
    target: &Path,
    on_progress: &(dyn Fn(u64, Option<u64>) + Send + Sync),
) -> Result<(), String> {
    use futures_util::StreamExt;
    use std::io::Write;
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let total = resp.content_length();
    let mut out = std::fs::File::create(target).map_err(|e| format!("创建文件失败: {e}"))?;
    let mut stream = resp.bytes_stream();
    let mut done: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取响应失败: {e}"))?;
        done += chunk.len() as u64;
        on_progress(done, total);
        out.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {e}"))?;
    }
    Ok(())
}

/// 下载 release 资产 zip（镜像优先：竞速选最快镜像下载，失败回退官方）
pub async fn download_release_zip(
    client: &reqwest::Client,
    repo: &str,
    version: &str,
    asset: &str,
    target: &Path,
    proxies: &[GithubProxy],
    on_progress: &(dyn Fn(u64, Option<u64>) + Send + Sync),
) -> Result<(), String> {
    // 镜像优先：并发竞速选最快镜像下载（镜像通常比官方快）
    if !proxies.is_empty() {
        let mut candidates = Vec::with_capacity(proxies.len());
        for p in proxies {
            candidates.push(build_proxy_url(p, repo, version, asset));
        }
        if let Ok(fastest) = pick_fastest(&candidates, None).await {
            if download_to(client, &fastest, target, on_progress)
                .await
                .is_ok()
            {
                return Ok(());
            }
        }
    }
    // 官方保底
    let official = format!("{GITHUB_DOWNLOAD_BASE}/{repo}/releases/download/v{version}/{asset}");
    download_to(client, &official, target, on_progress).await
}