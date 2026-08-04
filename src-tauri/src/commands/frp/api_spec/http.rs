//! HTTP 请求构造与发送（含重定向防护，设计文档 §7.6.6）
//!
//! 重定向防护：no-redirect 客户端 → 手动校验 Location 仅允许同域 → 最多 5 次防循环。

use super::super::{EndpointDef, Envelope};
use super::envelope;
use crate::log_debug;
use crate::log_error;

/// 最大响应体大小（1MB）
const MAX_RESPONSE_SIZE: usize = 1024 * 1024;

/// 最大重定向次数
const MAX_REDIRECT_HOPS: usize = 5;

/// 默认请求超时（毫秒）
const DEFAULT_TIMEOUT_MS: u64 = 10_000;

/// 构造并发送厂商 API 请求（含重定向防护 + envelope 成功校验）
///
/// - `tunnel_id`：当前隧道自增 ID，用于填充 query 中的 `{id}`/`{tunnel}` 模板。
/// - `tunnel_name`：当前隧道 name（真实隧道标识），填充 `{tunnelName}` 模板
///   （如 config 端点 `query: {"tunnel": "{tunnelName}"}`）。
///
/// 列表/账号类端点两者传空字符串。
///
/// 返回解析后的 JSON 响应。若 envelope 判断失败则返回错误。
#[allow(clippy::too_many_arguments)]
pub(super) async fn send_request(
    base_url: &str,
    endpoint: &EndpointDef,
    token: &str,
    device_id: &str,
    provider_id: &str,
    tunnel_id: &str,
    tunnel_name: &str,
    global_envelope: Option<&Envelope>,
) -> Result<serde_json::Value, String> {
    let url = build_url(base_url, &endpoint.path)?;
    let client = build_vendor_client()?;
    let method = endpoint.method.to_uppercase();
    let auth_value = build_auth_value(token);

    let allowed_host =
        extract_host(&url).ok_or_else(|| format!("无法解析 API URL 主机: {}", url))?;

    let mut current_url = url;
    let mut current_method = method.clone();
    let mut hops = 0usize;

    loop {
        let mut request = match current_method.as_str() {
            "GET" => client.get(&current_url),
            "POST" => client.post(&current_url),
            other => return Err(format!("不支持的 HTTP 方法: {}", other)),
        };

        if hops == 0 {
            // 首次请求：注入 token + query 参数
            request = inject_auth_and_params(
                request,
                &auth_value,
                &endpoint.query,
                &current_method,
                device_id,
                provider_id,
                tunnel_id,
                tunnel_name,
            )?;
        } else {
            // 重定向请求：仅重新注入 header 类 token
            request = request.header("Authorization", &auth_value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("厂商 API 请求发送失败: {}", e))?;

        if !response.status().is_redirection() {
            let value = handle_response(response).await?;

            // envelope 成功校验
            let effective_envelope = endpoint.envelope.as_ref().or(global_envelope);
            if !envelope::is_success(&value, effective_envelope) {
                let err_msg = envelope::extract_error(&value, effective_envelope)
                    .unwrap_or_else(|| "未知错误".to_string());
                log_error!(
                    "[Frp] 厂商 API 请求失败: {} {} - {}",
                    endpoint.method,
                    endpoint.path,
                    err_msg
                );
                return Err(format!("厂商 API 返回错误: {}", err_msg));
            }

            return Ok(value);
        }

        // 处理重定向
        hops += 1;
        if hops > MAX_REDIRECT_HOPS {
            return Err(format!(
                "厂商 API 重定向次数过多（超过 {} 次）",
                MAX_REDIRECT_HOPS
            ));
        }

        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                format!(
                    "厂商 API 返回重定向但无 Location 头: HTTP {}",
                    response.status()
                )
            })?;

        let next_url = resolve_url(&current_url, location)?;
        let next_host = extract_host(&next_url)
            .ok_or_else(|| format!("无法解析重定向 URL 主机: {}", next_url))?;

        if next_host != allowed_host {
            return Err(format!(
                "厂商 API 重定向到非白名单域名: {}（仅允许 {}）",
                next_host, allowed_host
            ));
        }

        log_debug!("[Frp] 厂商 API 重定向: {} -> {}", current_url, next_url);
        current_url = next_url;
        current_method = "GET".to_string();
    }
}

/// 构造认证值（Bearer token）
fn build_auth_value(token: &str) -> String {
    format!("Bearer {}", token)
}

/// 注入认证 token 和请求参数
#[allow(clippy::too_many_arguments)]
fn inject_auth_and_params(
    mut request: reqwest::RequestBuilder,
    auth_value: &str,
    query: &std::collections::HashMap<String, String>,
    method: &str,
    device_id: &str,
    provider_id: &str,
    tunnel_id: &str,
    tunnel_name: &str,
) -> Result<reqwest::RequestBuilder, String> {
    // token 注入到 Authorization header
    request = request.header("Authorization", auth_value);

    // 参数填充
    match method {
        "GET" => {
            for (k, v) in query {
                let filled = fill_template(v, device_id, provider_id, tunnel_id, tunnel_name);
                request = request.query(&[(k.as_str(), filled.as_str())]);
            }
        }
        "POST" => {
            // POST 时 query 仍作为 query string（部分厂商 POST 也用 query）
            for (k, v) in query {
                let filled = fill_template(v, device_id, provider_id, tunnel_id, tunnel_name);
                request = request.query(&[(k.as_str(), filled.as_str())]);
            }
        }
        _ => {}
    }

    Ok(request)
}

/// 填充参数模板中的上下文占位符
///
/// 支持：`{device_id}`、`{provider_id}`、`{id}`/`{tunnel}`（隧道自增 ID）、
/// `{tunnelName}`（隧道 name，真实隧道标识）。
#[allow(clippy::too_many_arguments)]
fn fill_template(
    template: &str,
    device_id: &str,
    provider_id: &str,
    tunnel_id: &str,
    tunnel_name: &str,
) -> String {
    template
        .replace("{device_id}", device_id)
        .replace("{provider_id}", provider_id)
        .replace("{tunnelName}", tunnel_name)
        .replace("{tunnel}", tunnel_id)
        .replace("{id}", tunnel_id)
}

/// 处理 HTTP 响应：状态码校验 + 大小限制 + JSON 解析
async fn handle_response(response: reqwest::Response) -> Result<serde_json::Value, String> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        // 非 2xx 不一定是致命错误：厂商 API 回填/探测类请求（config、detail）失败
        // 是预期内路径，调用方会自行决定是否报告，这里仅 debug 级记录完整信息，
        // 避免把"某厂商某端点预期失败"刷成 ERROR 误导排查。
        log_debug!(
            "[Frp] 厂商 API 非 2xx 响应: HTTP {} - {}",
            status,
            truncate(&body, 500)
        );
        return Err(format!(
            "厂商 API 请求失败: HTTP {} - {}",
            status,
            truncate(&body, 500)
        ));
    }

    if let Some(len) = response.content_length() {
        if len as usize > MAX_RESPONSE_SIZE {
            return Err(format!("厂商 API 响应过大: {} 字节（限制 1MB）", len));
        }
    }

    let body = response
        .bytes()
        .await
        .map_err(|e| format!("读取厂商 API 响应失败: {}", e))?;
    if body.len() > MAX_RESPONSE_SIZE {
        return Err(format!(
            "厂商 API 响应过大: {} 字节（限制 1MB）",
            body.len()
        ));
    }

    let value: serde_json::Value =
        serde_json::from_slice(&body).map_err(|e| format!("解析厂商 API 响应 JSON 失败: {}", e))?;

    Ok(value)
}

// URL 辅助函数
/// 拼接 baseUrl + path
fn build_url(base_url: &str, path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Ok(base_url.trim_end_matches('/').to_string());
    }
    if path.starts_with("http://") || path.starts_with("https://") {
        return Ok(path.to_string());
    }
    let base = base_url.trim_end_matches('/');
    let p = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    Ok(format!("{}{}", base, p))
}

/// 构建厂商 API 专用 HTTP 客户端（no-redirect + 复用全局代理/IP/TLS 信任源配置）
///
/// 复用 `crate::http` 的统一管线：User-Agent、代理、IP 协议版本偏好、
/// TLS 信任源（trust_mode / ignore_tls 开发者模式）与全局客户端一致；
/// 仅重定向策略设为 none，由上层手动校验 Location 域名白名单（设计文档 §7.6.6）。
fn build_vendor_client() -> Result<reqwest::Client, String> {
    Ok(crate::http::build_client_with_redirect(
        reqwest::redirect::Policy::none(),
        Some(DEFAULT_TIMEOUT_MS),
    ))
}

/// 从 URL 提取主机名（小写，不含端口）
fn extract_host(url: &str) -> Option<String> {
    let no_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host_end = no_scheme.find(['/', '?', '#']).unwrap_or(no_scheme.len());
    let host_with_port = &no_scheme[..host_end];
    let host = host_with_port.split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

/// 解析相对 Location 为绝对 URL
fn resolve_url(base: &str, location: &str) -> Result<String, String> {
    if location.starts_with("http://") || location.starts_with("https://") {
        return Ok(location.to_string());
    }
    let (scheme, no_scheme) = base
        .strip_prefix("https://")
        .map(|rest| ("https", rest))
        .or_else(|| base.strip_prefix("http://").map(|rest| ("http", rest)))
        .ok_or_else(|| format!("无效的基准 URL: {}", base))?;
    let host_end = no_scheme.find('/').unwrap_or(no_scheme.len());
    let host = &no_scheme[..host_end];
    if location.starts_with('/') {
        Ok(format!("{}://{}{}", scheme, host, location))
    } else {
        Ok(format!("{}://{}/{}", scheme, host, location))
    }
}

/// 截断字符串到指定长度
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
