//! GitHub 下载公共组件：release 资产下载（镜像优先 + 官方保底）
//! 供 easytier 内核等外部二进制按需下载复用。

use std::path::Path;
use std::sync::Arc;
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
        // full 模式：base 与完整 GitHub URL 之间补 `/`（base 可能无尾斜杠）
        format!("{base}/{GITHUB_DOWNLOAD_BASE}/{repo}/releases/download/v{version}/{asset}")
    }
}

/// 并发测速 URL 列表（HEAD + Range 0-1，单请求 10s 超时，禁止重定向），
/// 返回按耗时升序排列的可用列表；`cancel_flag` 置位时返回「下载已取消」。
///
/// 使用共享无重定向单例 `no_redirect_client()`：会跳转的镜像直接失败剔除
/// （重定向引入额外跳转，慢且不稳定）。
///
/// 逐候选 DEBUG 日志（发送的原始 URL + 状态/错误 + 耗时）：
/// 排查"full 模式被拼成 path"等 URL 形态问题，定位竞速为何全灭回退官方。
pub async fn probe_urls(
    urls: &[String],
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> Result<Vec<(Duration, String)>, String> {
    let client = crate::http::no_redirect_client();
    let mut handles = Vec::with_capacity(urls.len());
    for url in urls {
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
                    crate::log_debug!(
                        "[GitHub] 测速可用: {}ms status={} url={}",
                        start.elapsed().as_millis(),
                        r.status(),
                        url
                    );
                    Some((start.elapsed(), url))
                }
                Ok(r) => {
                    crate::log_debug!(
                        "[GitHub] 测速被拒: {}ms status={} url={}",
                        start.elapsed().as_millis(),
                        r.status(),
                        url
                    );
                    None
                }
                Err(e) => {
                    crate::log_debug!(
                        "[GitHub] 测速失败: {}ms err={} url={}",
                        start.elapsed().as_millis(),
                        crate::http::request_error_msg(&e),
                        url
                    );
                    None
                }
            }
        }));
    }
    let mut results = Vec::new();
    for mut h in handles {
        // 每 200ms 轮询取消信号，同时等待测速完成（&mut 借用避免 select 丢弃已完成结果）
        loop {
            if let Some(ref flag) = cancel_flag {
                if flag.load(std::sync::atomic::Ordering::Relaxed) {
                    return Err("下载已取消".to_string());
                }
            }
            match tokio::time::timeout(Duration::from_millis(200), &mut h).await {
                Ok(r) => {
                    if let Ok(Some((elapsed, url))) = r {
                        results.push((elapsed, url));
                    }
                    break;
                }
                Err(_) => continue,
            }
        }
    }
    results.sort_by_key(|(t, _)| *t);
    Ok(results)
}

/// 镜像竞速：并发测速取响应最快者（`probe_urls` 的取首封装）
pub async fn pick_fastest(
    candidates: &[String],
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> Result<String, String> {
    let results = probe_urls(candidates, cancel_flag).await?;
    match results.into_iter().next() {
        Some((_, url)) => {
            crate::log_debug!("[GitHub] 竞速胜者: {url}");
            Ok(url)
        }
        None => Err("所有镜像源均不可用".to_string()),
    }
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
