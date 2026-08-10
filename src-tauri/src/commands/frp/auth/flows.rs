//! 可配置认证流程引擎
//!
//! 按 endpoints.json 的 authFlows 配置构造 HTTP 请求并解析响应。

use super::super::log_redact::redact_log;
use super::super::types::{FieldExtractor, FlowRequest};
use crate::log_debug;
use std::collections::HashMap;

/// 占位符上下文（认证流程中所有可用变量）
#[derive(Debug, Clone, Default)]
pub struct FlowContext {
    pub base_url: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub code: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
    pub device_code: Option<String>,
    pub scope: Option<String>,
    pub api_key: Option<String>,
    pub public_key: Option<String>,
    pub request_uuid: Option<String>,
}

impl FlowContext {
    /// 按占位符名称取值
    fn get(&self, key: &str) -> Option<&str> {
        match key {
            "baseUrl" | "base_url" => self.base_url.as_deref(),
            "clientId" | "client_id" => Some(&self.client_id),
            "clientSecret" | "client_secret" => self.client_secret.as_deref(),
            "redirectUri" | "redirect_uri" => self.redirect_uri.as_deref(),
            "code" => self.code.as_deref(),
            "codeVerifier" | "code_verifier" => self.code_verifier.as_deref(),
            "refreshToken" | "refresh_token" => self.refresh_token.as_deref(),
            "deviceCode" | "device_code" => self.device_code.as_deref(),
            "scope" => self.scope.as_deref(),
            "apiKey" | "api_key" => self.api_key.as_deref(),
            "publicKey" | "public_key" => self.public_key.as_deref(),
            "requestUuid" | "request_uuid" => self.request_uuid.as_deref(),
            _ => None,
        }
    }
}

/// 发送可配置的认证流程请求
///
/// 按 FlowRequest 构造 HTTP 请求（method + url + contentType + body + headers），
/// 发送并返回响应（含状态码、headers、body）。
pub async fn send_flow_request(
    flow: &FlowRequest,
    ctx: &FlowContext,
) -> Result<FlowResponse, String> {
    let url = fill_template(&flow.url, ctx);
    let method = flow.method.to_uppercase();
    let content_type = if flow.content_type.is_empty() {
        "application/json"
    } else {
        &flow.content_type
    };
    let mut body_log: Option<String> = None;

    let client = crate::http::get_client();
    let mut request = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        other => return Err(format!("不支持的认证流程 HTTP 方法: {}", other)),
    };

    // 注入自定义 headers
    for (k, v) in &flow.headers {
        request = request.header(k, fill_template(v, ctx));
    }

    // POST body
    if method == "POST" {
        let body = fill_body_template(&flow.body, ctx);

        if content_type.contains("form-urlencoded") {
            // form-urlencoded：body 为对象，转为 key=value 对
            if let serde_json::Value::Object(map) = &body {
                let form_pairs: Vec<(String, String)> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), value_to_string(v)))
                    .collect();
                body_log = Some(
                    form_pairs
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&"),
                );
                request = request.form(&form_pairs);
            } else {
                body_log = Some(body.to_string());
                request = request.body(body.to_string());
            }
        } else {
            // JSON 或其他：直接发送 body
            body_log = Some(body.to_string());
            request = request
                .header("Content-Type", content_type)
                .body(body.to_string());
        }
    }

    log_debug!(
        "[Frp Auth] 认证流程请求: {} {} (contentType={}, body={})",
        method,
        url,
        content_type,
        redact_log(body_log.as_deref().unwrap_or(""))
    );

    let response = request
        .send()
        .await
        .map_err(|e| format!("认证流程请求失败: {}", crate::http::request_error_msg(&e)))?;

    let status = response.status();
    let headers = response.headers().clone();
    let body_text = response
        .text()
        .await
        .map_err(|e| format!("读取认证流程响应失败: {}", e))?;

    log_debug!(
        "[Frp Auth] 认证流程响应: HTTP {} - {}",
        status,
        redact_log(&body_text)
    );

    Ok(FlowResponse {
        status,
        headers,
        body: body_text,
    })
}

/// 认证流程响应
pub struct FlowResponse {
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: String,
}

impl FlowResponse {
    /// 判断 HTTP 状态是否成功
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// 按 FieldExtractor 从响应中取值
    ///
    /// from=body：解析 body 为 JSON，按 path（JSONPath）取值
    /// from=header：按 name 从响应头取值
    pub fn extract_field(&self, extractor: &FieldExtractor) -> Option<String> {
        match extractor.from.as_str() {
            "body" => {
                let path = extractor.path.as_deref()?;
                let value: serde_json::Value = serde_json::from_str(&self.body).ok()?;
                crate::commands::frp::api_spec::jsonpath::extract(&value, path).map(|v| match v {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    other => other.to_string(),
                })
            }
            "header" => {
                let name = extractor.name.as_deref()?;
                self.headers
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            }
            _ => None,
        }
    }

    /// 按提取规则批量取值
    #[allow(dead_code)]
    pub fn extract_all(
        &self,
        response_map: &HashMap<String, FieldExtractor>,
    ) -> HashMap<String, String> {
        response_map
            .iter()
            .filter_map(|(key, extractor)| self.extract_field(extractor).map(|v| (key.clone(), v)))
            .collect()
    }
}

/// 填充模板字符串中的占位符
fn fill_template(template: &str, ctx: &FlowContext) -> String {
    let mut result = template.to_string();
    // 支持的占位符列表
    let placeholders = [
        "baseUrl",
        "clientId",
        "clientSecret",
        "redirectUri",
        "code",
        "codeVerifier",
        "refreshToken",
        "deviceCode",
        "scope",
        "apiKey",
        "publicKey",
        "requestUuid",
    ];
    for ph in &placeholders {
        let pattern = format!("{{{}}}", ph);
        if result.contains(&pattern) {
            if let Some(val) = ctx.get(ph) {
                result = result.replace(&pattern, val);
            }
        }
    }
    result
}

/// 填充 body 模板（递归处理 Object/Array/String）
///
/// Object 中填充后仍残留未解析占位符（`{xxx}`）的字段会被删除：
/// 用于 PKCE 等场景——endpoints.json 声明 `client_secret: "{clientSecret}"`，
/// 而当前上下文无 client_secret 时该字段自动省略，而非发送字面量占位符。
fn fill_body_template(body: &serde_json::Value, ctx: &FlowContext) -> serde_json::Value {
    match body {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                let filled = fill_body_template(v, ctx);
                // 填充后仍含未解析占位符的字段删除（如无 client_secret 时的 {clientSecret}）
                if let serde_json::Value::String(s) = &filled {
                    if s.contains('{') && s.contains('}') {
                        continue;
                    }
                }
                new_map.insert(k.clone(), filled);
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(|v| fill_body_template(v, ctx)).collect())
        }
        serde_json::Value::String(s) => serde_json::Value::String(fill_template(s, ctx)),
        other => other.clone(),
    }
}

/// JSON Value → String（用于 form-urlencoded）
fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}

/// 从 FlowRequest.response 取指定字段的 FieldExtractor
///
/// 字段名按 camelCase 约定：accessToken / refreshToken / expiresIn /
/// errorField / errorDescription
pub(super) fn get_extractor<'a>(flow: &'a FlowRequest, key: &str) -> &'a FieldExtractor {
    static EMPTY: once_cell::sync::Lazy<FieldExtractor> =
        once_cell::sync::Lazy::new(|| FieldExtractor {
            from: "body".to_string(),
            path: None,
            name: None,
        });
    flow.response.get(key).unwrap_or(&EMPTY)
}

/// 从响应中提取错误消息（按 errorField / errorDescription 提取）
pub(super) fn extract_flow_error(resp: &FlowResponse, flow: &FlowRequest) -> String {
    let err = resp.extract_field(get_extractor(flow, "errorField"));
    let desc = resp.extract_field(get_extractor(flow, "errorDescription"));
    match (err, desc) {
        (Some(e), Some(d)) if !e.is_empty() && !d.is_empty() => format!("{}: {}", e, d),
        (Some(e), _) => e,
        _ => "未知错误".to_string(),
    }
}

#[cfg(test)]
#[path = "flows_tests.rs"]
mod tests;
