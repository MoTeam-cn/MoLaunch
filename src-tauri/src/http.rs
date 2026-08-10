//! HTTP 客户端模块：统一管理 reqwest 客户端构建，支持代理、IP 协议版本偏好与 TLS 信任源。

mod client;
mod tls;

pub use client::{
    build_client, build_client_with_redirect, build_client_with_user_agent, build_stream_client,
    ClientBuildParams,
};
pub use tls::ignore_tls_allowed;

use std::sync::{OnceLock, RwLock};
use std::time::Duration;

/// 全局 HTTP 客户端（可热重建）。
static HTTP_CLIENT: RwLock<Option<reqwest::Client>> = RwLock::new(None);

/// 编译时生成的 User-Agent 字符串（缓存）。
static USER_AGENT: OnceLock<String> = OnceLock::new();

fn user_agent() -> &'static str {
    USER_AGENT.get_or_init(crate::utils::client_type::user_agent)
}

/// 初始化或重建全局 HTTP 客户端。
///
/// 重复调用安全：直接覆盖旧客户端，进行中的请求仍使用旧客户端完成。
pub fn init_client(
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
    ip_version: &str,
    trust_mode: &str,
    ignore_tls: bool,
) {
    let client = build_client(
        proxy_mode,
        proxy_type,
        proxy_url,
        ip_version,
        Duration::from_secs(30),
        trust_mode,
        ignore_tls,
    );
    let mut guard = HTTP_CLIENT.write().expect("HTTP_CLIENT poisoned");
    *guard = Some(client);
}

/// 获取全局 HTTP 客户端。
///
/// 如果未初始化，返回一个无代理的默认客户端。
pub fn get_client() -> reqwest::Client {
    {
        let guard = HTTP_CLIENT.read().expect("HTTP_CLIENT poisoned");
        if let Some(ref client) = *guard {
            return client.clone();
        }
    }
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(user_agent())
        .no_proxy()
        .tls_built_in_root_certs(true)
        .build()
        .expect("Failed to build default HTTP client")
}

/// 错误链中是否包含 TLS 证书校验失败（抓包代理/中间人场景）
pub fn is_tls_cert_error(e: &(dyn std::error::Error + 'static)) -> bool {
    std::iter::successors(Some(e), |err| err.source()).any(|err| {
        let msg = err.to_string().to_ascii_lowercase();
        msg.contains("certificate")
            || msg.contains("unknownissuer")
            || msg.contains("tls handshake")
    })
}

/// 格式化请求错误：TLS 证书校验失败时提示中间人攻击
pub fn request_error_msg(e: &reqwest::Error) -> String {
    if is_tls_cert_error(e) {
        "检测到中间人攻击，已自动断开链接".to_string()
    } else {
        e.to_string()
    }
}

/// GET 请求并返回 (HTTP 状态码, 响应体文本)。
///
/// 网络错误返回 Err；HTTP 任意状态码均返回 Ok。
pub async fn get_text_with_status(url: &str) -> anyhow::Result<(u16, String)> {
    let resp = get_client()
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!(request_error_msg(&e)))?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok((status, text))
}

/// GET 请求并返回响应体文本，HTTP 非 2xx 时返回 Err。
pub async fn fetch_url(url: &str) -> anyhow::Result<String> {
    let (status, text) = get_text_with_status(url).await?;
    if !(200..300).contains(&status) {
        return Err(anyhow::anyhow!("HTTP error: {}", status));
    }
    Ok(text)
}

/// GET 请求并把响应体保存到文件。
pub async fn fetch_url_to_file(url: &str, local_path: &std::path::Path) -> anyhow::Result<String> {
    let content = fetch_url(url).await?;
    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(local_path, &content)?;
    Ok(content)
}

/// POST JSON 请求并返回 (HTTP 状态码, 响应体文本)。
pub async fn post_json_with_status<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> anyhow::Result<(u16, String)> {
    let resp = get_client()
        .post(url)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Accept-Language", "zh-CN")
        .json(body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!(request_error_msg(&e)))?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok((status, text))
}

#[cfg(test)]
#[path = "http_test.rs"]
mod http_test;
