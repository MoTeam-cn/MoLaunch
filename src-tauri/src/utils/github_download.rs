//! GitHub 下载公共组件：release 资产下载（镜像优先 + 官方保底）
//! 供 easytier 内核等外部二进制按需下载复用。

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

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

/// 构造镜像源下载 URL（type: path 追加路径 / type: full 追加完整 GitHub URL）
pub fn build_proxy_url(proxy: &GithubProxy, repo: &str, version: &str, asset: &str) -> String {
    let base = proxy.base.trim_end_matches('/');
    if proxy.proxy_type == "path" {
        format!("{base}/{repo}/releases/download/v{version}/{asset}")
    } else {
        format!("{base}{GITHUB_DOWNLOAD_BASE}/{repo}/releases/download/v{version}/{asset}")
    }
}

/// 镜像竞速：并发 HEAD + Range 0-1 测速（单请求 10s 超时），取响应最快者
pub async fn pick_fastest(
    client: &reqwest::Client,
    candidates: &[String],
) -> Result<String, String> {
    let mut handles = Vec::with_capacity(candidates.len());
    for url in candidates {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            let start = std::time::Instant::now();
            let resp = client
                .get(&url)
                .header("Range", "bytes=0-1")
                .timeout(Duration::from_secs(10))
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() || r.status().as_u16() == 206 => {
                    Some((start.elapsed(), url))
                }
                _ => None,
            }
        }));
    }
    let mut best: Option<(Duration, String)> = None;
    for h in handles {
        if let Ok(Some((elapsed, url))) = h.await {
            if best.as_ref().map(|(t, _)| elapsed < *t).unwrap_or(true) {
                best = Some((elapsed, url));
            }
        }
    }
    best.map(|(_, url)| url)
        .ok_or_else(|| "所有镜像源均不可用".to_string())
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
        if let Ok(fastest) = pick_fastest(client, &candidates).await {
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
