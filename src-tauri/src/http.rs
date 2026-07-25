//! HTTP 客户端模块
//! 统一管理 reqwest 客户端构建，支持代理配置

use std::sync::OnceLock;
use std::time::Duration;

/// 全局 HTTP 客户端（懒初始化）
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// 编译时生成的 User-Agent 字符串
/// 格式：`MoLaunch/<os> <version>`（如 `MoLaunch/windows 0.1.0`）
/// - `<os>` 来自 `std::env::consts::OS`（编译期不可用，运行时取值）
/// - `<version>` 来自 Cargo.toml 的 `version` 字段（编译期通过 `env!` 注入）
static USER_AGENT: OnceLock<String> = OnceLock::new();

/// 获取 User-Agent 字符串（运行时拼接 OS + 编译时版本号）
fn user_agent() -> &'static str {
    USER_AGENT.get_or_init(|| {
        format!(
            "MoLaunch/{} {}",
            std::env::consts::OS,
            env!("CARGO_PKG_VERSION")
        )
    })
}

/// 初始化全局 HTTP 客户端
/// 应在应用启动时调用一次
pub fn init_client(proxy_mode: &str, proxy_type: &str, proxy_url: &str) {
    let client = build_client(proxy_mode, proxy_type, proxy_url, Duration::from_secs(30));
    HTTP_CLIENT
        .set(client)
        .expect("HTTP client already initialized");
}

/// 获取全局 HTTP 客户端
/// 如果未初始化，返回一个无代理的默认客户端
pub fn get_client() -> reqwest::Client {
    HTTP_CLIENT.get().cloned().unwrap_or_else(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(user_agent())
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
    let mut builder = reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(user_agent());

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

/// GET 请求并返回 (HTTP 状态码, 响应体文本)
///
/// 与 `fetch_url` 不同，本函数保留状态码信息，便于调用方按状态码做差异化错误处理
/// （如 yggdrasil 协议中 204 表示 validate 成功，403 表示 token 失效）。
///
/// 网络错误（无法连接服务器）时返回 Err；HTTP 任意状态码（含 4xx/5xx）均返回 Ok。
pub async fn get_text_with_status(url: &str) -> anyhow::Result<(u16, String)> {
    let client = get_client();
    let resp = client.get(url).send().await?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok((status, text))
}

/// POST JSON 请求并返回 (HTTP 状态码, 响应体文本)
///
/// 统一的 POST JSON 入口，自动设置 `Content-Type: application/json; charset=utf-8`
/// 和 `Accept-Language: zh-CN` 请求头。
/// 与 `fetch_url` 不同，本函数保留状态码信息，便于调用方按状态码做差异化错误处理。
///
/// 网络错误（无法连接服务器）时返回 Err；HTTP 任意状态码（含 4xx/5xx）均返回 Ok。
pub async fn post_json_with_status<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> anyhow::Result<(u16, String)> {
    let client = get_client();
    let resp = client
        .post(url)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Accept-Language", "zh-CN")
        .json(body)
        .send()
        .await?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok((status, text))
}

/// GET 请求并返回二进制内容
///
/// 用于下载 jar/图片等二进制资源。HTTP 非 2xx 返回 Err。
pub async fn fetch_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let client = get_client();
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", resp.status()));
    }
    Ok(resp.bytes().await?.to_vec())
}
