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

/// 镜像竞速：并发 HEAD + Range 0-1 测速（单请求 10s 超时，禁止重定向），取响应最快者
///
/// 使用共享无重定向单例 `no_redirect_client()`：会跳转的镜像直接失败剔除
/// （重定向引入额外跳转，慢且不稳定）。
///
/// 逐候选 DEBUG 日志（发送的原始 URL + 状态/错误 + 耗时 + 最终胜者）：
/// 排查"full 模式被拼成 path"等 URL 形态问题，定位竞速为何全灭回退官方。
///
/// `cancel_flag`：等待测速期间每 200ms 轮询取消信号，取消立即中断竞速（返回「下载已取消」），
/// 避免挂起镜像（10s 超时）拖住取消响应。
pub async fn pick_fastest(
    candidates: &[String],
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> Result<String, String> {
    let client = crate::http::no_redirect_client();
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
                    crate::log_debug!(
                        "[EasyTier] 竞速探测可用: {}ms status={} url={}",
                        start.elapsed().as_millis(),
                        r.status(),
                        url
                    );
                    Some((start.elapsed(), url))
                }
                Ok(r) => {
                    crate::log_debug!(
                        "[EasyTier] 竞速探测被拒: {}ms status={} url={}",
                        start.elapsed().as_millis(),
                        r.status(),
                        url
                    );
                    None
                }
                Err(e) => {
                    crate::log_debug!(
                        "[EasyTier] 竞速探测失败: {}ms err={} url={}",
                        start.elapsed().as_millis(),
                        crate::http::request_error_msg(&e),
                        url
                    );
                    None
                }
            }
        }));
    }
    let mut best: Option<(Duration, String)> = None;
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
                        if best.as_ref().map(|(t, _)| elapsed < *t).unwrap_or(true) {
                            best = Some((elapsed, url));
                        }
                    }
                    break;
                }
                Err(_) => continue,
            }
        }
    }
    match best {
        Some((_, url)) => {
            crate::log_debug!("[EasyTier] 竞速胜者: {url}");
            Ok(url)
        }
        None => Err("所有镜像源均不可用".to_string()),
    }
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
