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

/// 流式请求的通用超时包装：在 `timeout_secs`（下限 5s）内完成，返回响应体供逐块读取
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
