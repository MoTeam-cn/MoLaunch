//! HTTP 传输层：认证请求构建 + 请求级超时
//!
//! 复用 `crate::http` 全局客户端（代理 / IP 协议版本 / TLS 信任源随配置热更新，
//! 修改后由 `apply_config` 重建全局客户端，本模块通过 `http::get_client()` 获取
//! 最新快照，无需重启应用即可生效）。

use crate::ai_core::config::AiConfig;

/// 构造带可选 `Authorization: Bearer <api_key>` 的请求构建器
pub(crate) fn authorized_builder(
    config: &AiConfig,
    method: reqwest::Method,
    url: String,
) -> reqwest::RequestBuilder {
    let mut builder = crate::http::get_client()
        .request(method, url)
        .header("Accept-Language", "zh-CN");
    if !config.api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {}", config.api_key));
    }
    builder
}

/// 构造带可选 `Authorization: Bearer <api_key>` 的**流式专用**请求构建器
///
/// 流式请求（SSE）可能持续数分钟（思考型模型首 token 即可达数十秒~数分钟），
/// 而全局 HTTP 客户端自带 30s 客户端级超时，会误杀仍在思考的流式请求。
/// 这里用与全局客户端相同的代理 / IP 版本 / TLS 管线重建一个**无整体超时**的
/// 客户端（仅流式链路使用），连接/首字节等待由 `send_stream_with_timeout` 控制。
pub(crate) fn authorized_stream_builder(
    config: &AiConfig,
    method: reqwest::Method,
    url: String,
) -> reqwest::RequestBuilder {
    let mut builder = stream_client()
        .request(method, url)
        .header("Accept-Language", "zh-CN");
    if !config.api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {}", config.api_key));
    }
    builder
}

/// 流式专用 HTTP 客户端：复用全局客户端同款管线（代理 / IP 版本 / TLS 信任源），
/// 但不设置客户端级整体超时（timeout(None)），避免思考型模型长首 token 被误杀。
fn stream_client() -> reqwest::Client {
    let config = crate::config::load_config().ok().flatten();
    let (mode, kind, url, ip_version, trust_mode) = config
        .as_ref()
        .map(|c| {
            (
                c.proxy.mode.clone(),
                c.proxy.kind.clone(),
                c.proxy.url.clone(),
                c.proxy.ip_version.clone(),
                c.tls.trust_mode.clone(),
            )
        })
        .unwrap_or_else(|| {
            (
                "none".to_string(),
                "http".to_string(),
                String::new(),
                "auto".to_string(),
                "builtin".to_string(),
            )
        });
    let ignore_tls = crate::commands::system::developer::is_ignore_tls();
    crate::http::build_stream_client(&mode, &kind, &url, &ip_version, &trust_mode, ignore_tls)
}

/// 通用超时包装：限制外部 future 在 `timeout_secs`（下限 5s）内完成
///
/// `crate::http` 全局客户端自带 30s 超时，此处提供可配置的请求级超时。
pub(crate) async fn send_with_timeout<F>(timeout_secs: u64, fut: F) -> anyhow::Result<(u16, String)>
where
    F: std::future::Future<Output = anyhow::Result<(u16, String)>> + Send,
{
    let timeout = std::time::Duration::from_secs(timeout_secs.max(5));
    tokio::time::timeout(timeout, fut)
        .await
        .map_err(|_| anyhow::anyhow!("AI 请求超时（{}s）", timeout.as_secs()))?
}

/// 流式请求的通用超时包装：在 `timeout_secs`（下限 5s）内返回响应体（首字节到达前），
/// 供逐块读取。注意：只约束「响应头到达」这一阶段——SSE 正文的长时间读取由
/// 流式专用客户端（无整体超时）承载，思考型模型长首 token 不会在此被误杀。
pub(crate) async fn send_stream_with_timeout<F>(
    timeout_secs: u64,
    fut: F,
) -> anyhow::Result<(u16, reqwest::Response)>
where
    F: std::future::Future<Output = anyhow::Result<(u16, reqwest::Response)>> + Send,
{
    let timeout = std::time::Duration::from_secs(timeout_secs.max(5));
    tokio::time::timeout(timeout, fut)
        .await
        .map_err(|_| anyhow::anyhow!("AI 请求超时（{}s）", timeout.as_secs()))?
}
