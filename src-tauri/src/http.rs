//! HTTP 客户端模块
//! 统一管理 reqwest 客户端构建，支持代理配置

use std::sync::OnceLock;
use std::time::Duration;

/// 全局 HTTP 客户端（懒初始化）
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// 初始化全局 HTTP 客户端
/// 应在应用启动时调用一次
pub fn init_client(proxy_mode: &str, proxy_type: &str, proxy_url: &str) {
    let client = build_client(proxy_mode, proxy_type, proxy_url, Duration::from_secs(30));
    HTTP_CLIENT.set(client).expect("HTTP client already initialized");
}

/// 获取全局 HTTP 客户端
/// 如果未初始化，返回一个无代理的默认客户端
pub fn get_client() -> reqwest::Client {
    HTTP_CLIENT.get().cloned().unwrap_or_else(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .no_proxy()
            .build()
            .expect("Failed to build default HTTP client")
    })
}

/// 构建 HTTP 客户端
pub fn build_client(
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
    timeout: Duration,
) -> reqwest::Client {
    let mut builder = reqwest::Client::builder().timeout(timeout);

    match proxy_mode {
        "system" => {
            // reqwest 默认使用系统代理，无需额外配置
        }
        "custom" => {
            if !proxy_url.is_empty() {
                let full_url = match proxy_type {
                    "socks5" => format!("socks5://{}", proxy_url),
                    "https" => format!("https://{}", proxy_url),
                    _ => format!("http://{}", proxy_url),
                };
                if let Ok(proxy) = reqwest::Proxy::all(&full_url) {
                    builder = builder.proxy(proxy);
                }
            }
        }
        _ => {
            // "none" - 不使用代理
            builder = builder.no_proxy();
        }
    }

    builder.build().expect("Failed to build HTTP client")
}

/// 获取 URL 内容（使用全局客户端）
pub async fn fetch_url(url: &str) -> anyhow::Result<String> {
    let client = get_client();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    Ok(response.text().await?)
}

/// 获取 URL 内容并保存到文件
pub async fn fetch_url_to_file(url: &str, local_path: &std::path::Path) -> anyhow::Result<String> {
    let client = get_client();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    let content = response.text().await?;

    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(local_path, &content)?;
    Ok(content)
}
