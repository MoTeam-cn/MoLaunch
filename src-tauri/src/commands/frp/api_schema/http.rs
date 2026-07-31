//! HTTP 请求构造与发送（含重定向防护，设计文档 §7.6.6）
//! 重定向防护：`redirect::Policy::none()` 禁止自动跟随 → 手动校验 Location 头仅允许
//! 与 baseUrl 同域的重定向 → 最多 5 次防循环。不使用 `crate::http::get_client()`
//! （其 redirect policy 不可覆盖），改用独立构建的 no-redirect 客户端，保留内置根证书。

use super::helpers::{
    build_url, build_vendor_client, compute_timeout, extract_host, fill_param_template, resolve_url,
    truncate,
};
use super::{ApiEndpoint, ApiSchema, AuthInjection, MAX_REDIRECT_HOPS, MAX_RESPONSE_SIZE};
use crate::log_debug;
use crate::log_error;

/// 构造并发送厂商 API 请求（含重定向防护）
pub(super) async fn send_api_request(
    schema: &ApiSchema,
    endpoint: &ApiEndpoint,
    token: &str,
    device_id: &str,
    provider_id: &str,
) -> Result<serde_json::Value, String> {
    let url = build_url(&schema.base_url, &endpoint.path)?;
    let timeout = compute_timeout(schema.timeout);
    let client = build_vendor_client(timeout)?;

    let method = endpoint.method.to_uppercase();
    let auth_value = schema
        .auth_injection
        .value_template
        .replace("{token}", token);

    let params = endpoint.params.clone().unwrap_or_default();
    let filled_params: Vec<(String, String)> = params
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                fill_param_template(&v.template, device_id, provider_id),
            )
        })
        .collect();

    let allowed_host = extract_host(&url)
        .ok_or_else(|| format!("无法解析 API URL 主机: {}", url))?;

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
            // 首次请求：注入 token + 填充参数
            request = inject_auth_and_params(
                request,
                &schema.auth_injection,
                &auth_value,
                &filled_params,
                &current_method,
            )?;
        } else if schema.auth_injection.location == "header" {
            // 重定向请求：仅重新注入 header 类 token（query/body 不重带，遵循 HTTP 语义）
            if let Some(ref name) = schema.auth_injection.header_name {
                request = request.header(name, &auth_value);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("厂商 API 请求发送失败: {}", e))?;

        if !response.status().is_redirection() {
            return handle_response(response).await;
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
        current_method = "GET".to_string(); // 重定向后转为 GET（HTTP 语义）
    }
}

/// 注入认证 token 和请求参数到 RequestBuilder
///
/// RequestBuilder 的 builder 方法消耗 self，因此按值接收并返回。
fn inject_auth_and_params(
    mut request: reqwest::RequestBuilder,
    auth_injection: &AuthInjection,
    auth_value: &str,
    params: &[(String, String)],
    method: &str,
) -> Result<reqwest::RequestBuilder, String> {
    // token 注入
    match auth_injection.location.as_str() {
        "header" => {
            let name = auth_injection
                .header_name
                .as_ref()
                .ok_or("auth_injection.location=header 但未提供 header_name")?;
            request = request.header(name, auth_value);
        }
        "query" => {
            let name = auth_injection
                .query_name
                .as_ref()
                .ok_or("auth_injection.location=query 但未提供 query_name")?;
            request = request.query(&[(name.as_str(), auth_value)]);
        }
        "body" => {
            // body 注入在 POST body 构造时处理
        }
        other => {
            return Err(format!("不支持的 auth_injection.location: {}", other));
        }
    }

    // 参数填充
    match method {
        "GET" => {
            for (k, v) in params {
                request = request.query(&[(k.as_str(), v.as_str())]);
            }
        }
        "POST" => {
            let mut body = serde_json::Map::new();
            for (k, v) in params {
                body.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            if auth_injection.location == "body" {
                let field = auth_injection
                    .body_field
                    .as_ref()
                    .ok_or("auth_injection.location=body 但未提供 body_field")?;
                body.insert(field.clone(), serde_json::Value::String(auth_value.to_string()));
            }
            request = request.json(&serde_json::Value::Object(body));
        }
        _ => unreachable!(),
    }

    Ok(request)
}

/// 处理 HTTP 响应：状态码校验 + 大小限制 + JSON 解析
async fn handle_response(response: reqwest::Response) -> Result<serde_json::Value, String> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log_error!("[Frp] 厂商 API 请求失败: HTTP {}", status);
        return Err(format!(
            "厂商 API 请求失败: HTTP {} - {}",
            status,
            truncate(&body, 500)
        ));
    }

    // 响应大小限制（Content-Length 头预检）
    if let Some(len) = response.content_length() {
        if len as usize > MAX_RESPONSE_SIZE {
            return Err(format!(
                "厂商 API 响应过大: {} 字节（限制 1MB）",
                len
            ));
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

    let value: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| format!("解析厂商 API 响应 JSON 失败: {}", e))?;

    Ok(value)
}
