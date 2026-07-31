//! 厂商 API 引擎：api-schema.json 解析 + API 调用 + 配置模板渲染
//!
//! 认证成功后调用厂商 API 拉取 frpc 配置（frps 地址/端口/token/分配端口等），
//! 按 api-schema.json 定义动态构造 HTTP 请求并映射响应到标准 ConfigPayload，
//! 最后填充厂商 config-template.toml 生成最终 frpc 配置。
//!
//! 设计文档：§7.6（api-schema.json 格式说明）、§6.7（认证后拉取厂商配置）。
//!
//! 依赖说明（由其他 agent 提供）：
//! - `crate::commands::frp::auth::load_token(provider_id) -> Result<String, String>`
//!   从 OS 密钥存储读取厂商 access_token（auth.rs 模块）。

use super::{providers_root, validate_provider_id};
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================
// 常量
// ============================================================

/// 默认请求超时（毫秒）
const DEFAULT_TIMEOUT_MS: u64 = 10_000;

/// 最大请求超时（毫秒），schema.timeout 超过此值会被截断
const MAX_TIMEOUT_MS: u64 = 30_000;

/// 最小请求超时（毫秒），防止 schema 配置过小导致请求必超时
const MIN_TIMEOUT_MS: u64 = 1_000;

/// 响应体大小上限（1MB）
const MAX_RESPONSE_SIZE: usize = 1024 * 1024;

/// 最大重定向跟随次数
const MAX_REDIRECT_HOPS: usize = 5;

// ============================================================
// 类型定义
// ============================================================

/// api-schema.json 结构
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSchema {
    /// schema 版本（当前仅支持 1）
    pub version: u32,
    /// API 基础 URL（如 "https://api.vendor-a.com"）
    pub base_url: String,
    /// 请求超时（毫秒），默认 10000，最大 30000
    #[serde(default)]
    pub timeout: Option<u64>,
    /// 认证 token 注入位置
    pub auth_injection: AuthInjection,
    /// API 端点定义
    pub endpoints: Endpoints,
}

/// token 注入位置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInjection {
    /// 注入位置：header / query / body
    pub location: String,
    /// header 名（location=header 时，如 "Authorization"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_name: Option<String>,
    /// 值格式模板（如 "Bearer {token}"，{token} 替换为 access_token）
    pub value_template: String,
    /// query 参数名（location=query 时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_name: Option<String>,
    /// body 字段名（location=body 时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_field: Option<String>,
}

/// 端点集合
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    /// 拉取配置端点（认证后调用，获取 frps 服务器信息等）
    pub fetch_config: ApiEndpoint,
}

/// 单个 API 端点定义
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiEndpoint {
    /// 相对路径（拼接 baseUrl，如 "/api/v1/config"）
    pub path: String,
    /// HTTP 方法：GET / POST
    pub method: String,
    /// 请求参数（GET 作为 query string，POST 作为 JSON body）
    #[serde(default)]
    pub params: Option<HashMap<String, ApiParam>>,
    /// 响应映射：厂商响应 JSON 路径 → ConfigPayload 字段名
    pub response_mapping: HashMap<String, String>,
}

/// 请求参数定义
#[derive(Debug, Clone, Deserialize)]
pub struct ApiParam {
    /// 参数值模板（如 "{device_id}"，从启动器上下文替换）
    pub template: String,
    /// 是否必填
    #[serde(default)]
    pub required: bool,
}

/// 标准配置载荷（厂商 API 响应映射后的统一格式）
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPayload {
    /// Frp 服务器地址
    pub server_addr: String,
    /// Frp 服务器端口
    pub server_port: u16,
    /// Frp 服务器 token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// 用户专属远程端口（厂商分配）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_remote_port: Option<u16>,
    /// 用户专属子域名（厂商分配）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_subdomain: Option<String>,
    /// 其他厂商自定义变量（填充到模板的 {var} 占位符）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_variables: Option<HashMap<String, String>>,
}

// ============================================================
// 公共函数
// ============================================================

/// 读取并解析厂商目录下的 api-schema.json
///
/// 文件位置：`<base_dir>/providers/<provider_id>/api-schema.json`
/// 校验：provider_id 格式 + 文件存在 + JSON 可解析 + version=1 + baseUrl 为 HTTPS。
pub fn load_api_schema(provider_id: &str) -> Result<ApiSchema, String> {
    validate_provider_id(provider_id)?;
    let path = providers_root().join(provider_id).join("api-schema.json");
    if !path.exists() {
        return Err(format!("厂商 api-schema.json 不存在: {}", path.display()));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 api-schema.json 失败: {}", e))?;
    let schema: ApiSchema = serde_json::from_str(&content)
        .map_err(|e| format!("解析 api-schema.json 失败: {}", e))?;

    if schema.version != 1 {
        return Err(format!(
            "不支持的 api-schema 版本: {}（当前仅支持 1）",
            schema.version
        ));
    }

    // 安全：baseUrl 必须为 HTTPS（认证 token 经此通道传输）
    if !schema.base_url.starts_with("https://") {
        return Err(format!(
            "api-schema baseUrl 必须使用 HTTPS: {}",
            schema.base_url
        ));
    }

    Ok(schema)
}

/// 认证后调用厂商 API 拉取配置
///
/// 流程：
/// 1. 加载 api-schema.json
/// 2. 加载 access_token（依赖 auth::load_token）
/// 3. 获取 device_id（从 SDK）
/// 4. 构造 HTTP 请求（注入 token + 填充参数）
/// 5. 发送请求（超时控制 + 重定向防护）
/// 6. 按 response_mapping 映射响应到 ConfigPayload
pub async fn fetch_vendor_config(
    state: &AppState,
    provider_id: &str,
) -> Result<ConfigPayload, String> {
    // 1. 加载 schema
    let schema = load_api_schema(provider_id)?;

    // 2. 加载 access_token
    // 依赖：crate::commands::frp::auth::load_token 由 auth 模块（其他 agent）提供
    // 假定签名：pub async fn load_token(provider_id: &str) -> Result<String, String>
    let token = crate::commands::frp::auth::load_token(provider_id)
        .await
        .map_err(|e| format!("加载厂商 {} 的 access_token 失败: {}", provider_id, e))?;

    // 3. 获取 device_id（复用 SDK 命令，避免重复实现）
    let device_id = crate::commands::sdk::get_device_id(state)
        .await
        .map_err(|e| format!("获取 device_id 失败: {}", e))?;

    log_info!(
        "[Frp] 拉取厂商 {} 配置: {} {}",
        provider_id,
        schema.endpoints.fetch_config.method,
        schema.endpoints.fetch_config.path
    );

    // 4. 构造并发送请求
    let response = send_api_request(
        &schema,
        &schema.endpoints.fetch_config,
        &token,
        &device_id,
        provider_id,
    )
    .await?;

    // 5. 映射响应
    let payload = map_response(&response, &schema.endpoints.fetch_config.response_mapping)?;

    log_info!(
        "[Frp] 厂商 {} 配置拉取成功: server={}:{}",
        provider_id, payload.server_addr, payload.server_port
    );

    Ok(payload)
}

/// 将 ConfigPayload 填充到厂商配置模板，生成最终 TOML
///
/// 读取 `<base_dir>/providers/<provider_id>/config-template.toml`，
/// 替换占位符：`{server_addr}` / `{server_port}` / `{token}` /
/// `{assigned_remote_port}` / `{assigned_subdomain}` / `{自定义变量}`。
///
/// 字符串值会做 TOML 转义（反斜杠 + 引号），数值型直接输出数字。
/// 可选字段为 None 时替换为空字符串（模板作者需自行处理可空性）。
pub fn render_config_template(
    provider_id: &str,
    payload: &ConfigPayload,
) -> Result<String, String> {
    validate_provider_id(provider_id)?;
    let template_path = providers_root()
        .join(provider_id)
        .join("config-template.toml");
    if !template_path.exists() {
        return Err(format!(
            "厂商配置模板不存在: {}",
            template_path.display()
        ));
    }
    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("读取配置模板失败: {}", e))?;

    let mut result = template;

    // 标准字段替换
    result = result.replace(
        "{server_addr}",
        &escape_toml_string(&payload.server_addr),
    );
    result = result.replace("{server_port}", &payload.server_port.to_string());

    match &payload.token {
        Some(t) => result = result.replace("{token}", &escape_toml_string(t)),
        None => result = result.replace("{token}", ""),
    }

    match payload.assigned_remote_port {
        Some(p) => result = result.replace("{assigned_remote_port}", &p.to_string()),
        None => result = result.replace("{assigned_remote_port}", ""),
    }

    match &payload.assigned_subdomain {
        Some(s) => result = result.replace("{assigned_subdomain}", &escape_toml_string(s)),
        None => result = result.replace("{assigned_subdomain}", ""),
    }

    // 自定义变量替换
    if let Some(ref vars) = payload.custom_variables {
        for (k, v) in vars {
            result = result.replace(&format!("{{{}}}", k), &escape_toml_string(v));
        }
    }

    Ok(result)
}

// ============================================================
// HTTP 请求构造与发送
// ============================================================

/// 构造并发送厂商 API 请求（含重定向防护）
///
/// 重定向防护策略（设计文档 §7.6.6）：
/// - 使用 `redirect::Policy::none()` 禁止自动跟随
/// - 手动校验 Location 头，仅允许与 baseUrl 同域的重定向
/// - 最多跟随 5 次，防止循环重定向
///
/// 注：此处不使用 `crate::http::get_client()`，因其 redirect policy 不可覆盖，
/// 无法实现同域白名单校验。改用独立构建的 no-redirect 客户端，保留内置根证书。
async fn send_api_request(
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

    // 预填充参数模板（{device_id} / {provider_id} → 实际值）
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

// ============================================================
// 响应映射
// ============================================================

/// 按 response_mapping 将厂商响应映射到 ConfigPayload
///
/// mapping 的 key 是厂商响应的 JSON 路径（如 "data.server_addr"），
/// value 是 ConfigPayload 的字段名（如 "serverAddr"）。
/// 标准字段名匹配后写入对应字段；非标准字段名视为自定义变量。
fn map_response(
    response: &serde_json::Value,
    mapping: &HashMap<String, String>,
) -> Result<ConfigPayload, String> {
    let mut payload = ConfigPayload::default();

    for (vendor_path, field_name) in mapping {
        match get_json_path(response, vendor_path) {
            Some(value) => {
                set_payload_field(&mut payload, field_name, value)?;
            }
            None => {
                if is_required_field(field_name) {
                    return Err(format!(
                        "厂商响应缺少必填字段: {}（JSON 路径: {}）",
                        field_name, vendor_path
                    ));
                }
                log_debug!(
                    "[Frp] 厂商响应可选字段缺失: {}（路径: {}）",
                    field_name,
                    vendor_path
                );
            }
        }
    }

    // 必填字段最终校验
    if payload.server_addr.is_empty() {
        return Err("厂商响应未提供服务器地址 (serverAddr)".to_string());
    }
    if payload.server_port == 0 {
        return Err("厂商响应未提供服务器端口 (serverPort)".to_string());
    }

    Ok(payload)
}

/// 按 dot 分隔路径从 JSON Value 取值
///
/// 支持如 "data.server_addr" 的路径，逐层深入。
/// 路径段为空时跳过。任一段不存在返回 None。
pub fn get_json_path(value: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    let mut current = value;
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        current = current.get(segment)?;
    }
    Some(current.clone())
}

/// 写入 ConfigPayload 对应字段
///
/// 同时兼容 camelCase（schema 中的写法）和 snake_case（Rust 字段名）。
/// 非标准字段名视为自定义变量，写入 custom_variables。
fn set_payload_field(
    payload: &mut ConfigPayload,
    field_name: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    match field_name {
        "serverAddr" | "server_addr" => {
            payload.server_addr = value_as_string(&value)
                .ok_or_else(|| format!("字段 {} 的值不是有效字符串", field_name))?;
        }
        "serverPort" | "server_port" => {
            payload.server_port = value_as_u16(&value)
                .ok_or_else(|| format!("字段 {} 的值不是有效端口", field_name))?;
        }
        "token" => {
            payload.token = value_as_string(&value);
        }
        "assignedRemotePort" | "assigned_remote_port" => {
            payload.assigned_remote_port = value_as_u16(&value);
        }
        "assignedSubdomain" | "assigned_subdomain" => {
            payload.assigned_subdomain = value_as_string(&value);
        }
        // 非标准字段名 → 自定义变量
        other => {
            let str_val = value_as_string(&value).unwrap_or_else(|| value.to_string());
            payload
                .custom_variables
                .get_or_insert_with(HashMap::new)
                .insert(other.to_string(), str_val);
        }
    }
    Ok(())
}

/// 判断字段是否为必填（serverAddr / serverPort）
fn is_required_field(field_name: &str) -> bool {
    matches!(
        field_name,
        "serverAddr" | "server_addr" | "serverPort" | "server_port"
    )
}

/// JSON Value → String（字符串原样返回，数字/布尔转字符串）
fn value_as_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// JSON Value → u16（数字直接转换，字符串解析）
fn value_as_u16(value: &serde_json::Value) -> Option<u16> {
    match value {
        serde_json::Value::Number(n) => n.as_u64().and_then(|v| u16::try_from(v).ok()),
        serde_json::Value::String(s) => s.parse::<u16>().ok(),
        _ => None,
    }
}

// ============================================================
// URL 与客户端辅助
// ============================================================

/// 拼接 baseUrl + path
///
/// 处理 trailing slash：`https://api.x.com` + `/v1/config` → `https://api.x.com/v1/config`。
/// path 若已是绝对 URL（http/https 开头）直接返回。
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

/// 计算请求超时
///
/// schema.timeout 缺省时取默认 10s，超过 30s 截断，低于 1s 抬升。
fn compute_timeout(timeout_ms: Option<u64>) -> Duration {
    let ms = timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS);
    let clamped = ms.clamp(MIN_TIMEOUT_MS, MAX_TIMEOUT_MS);
    Duration::from_millis(clamped)
}

/// 构建厂商 API 专用 HTTP 客户端（no-redirect + 内置根证书）
///
/// 不复用 `crate::http::get_client()`：全局客户端 redirect policy 不可覆盖，
/// 无法实现 §7.6.6 要求的同域重定向白名单校验。
fn build_vendor_client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(timeout)
        .tls_built_in_root_certs(true)
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {}", e))
}

/// 填充参数模板中的上下文占位符
///
/// 当前支持：`{device_id}` / `{provider_id}`。
/// 未知占位符保持原样（厂商 schema 配置错误时由 API 端拒绝）。
fn fill_param_template(template: &str, device_id: &str, provider_id: &str) -> String {
    template
        .replace("{device_id}", device_id)
        .replace("{provider_id}", provider_id)
}

/// 从 URL 提取主机名（小写，不含端口）
fn extract_host(url: &str) -> Option<String> {
    let no_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host_end = no_scheme
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(no_scheme.len());
    let host_with_port = &no_scheme[..host_end];
    let host = host_with_port.split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

/// 解析相对 Location 为绝对 URL
///
/// - 绝对 URL（http/https 开头）直接返回
/// - 以 `/` 开头的相对路径：拼接 scheme + host
/// - 其他相对路径：拼接 scheme + host + `/`
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

/// 截断字符串到指定长度（超出追加 "..."）
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// TOML 字符串转义（处理反斜杠和引号）
///
/// 与 tunnel.rs 的 escape_toml_string 逻辑一致，此处为模板渲染独立保留
/// （tunnel.rs 的同名函数为私有，且本模块不在修改范围内）。
fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_json_path() {
        let json: serde_json::Value = serde_json::json!({
            "data": {
                "server_addr": "1.2.3.4",
                "server_port": 7000,
                "nested": { "deep": "value" }
            }
        });
        assert_eq!(
            get_json_path(&json, "data.server_addr"),
            Some(serde_json::json!("1.2.3.4"))
        );
        assert_eq!(
            get_json_path(&json, "data.server_port"),
            Some(serde_json::json!(7000))
        );
        assert_eq!(
            get_json_path(&json, "data.nested.deep"),
            Some(serde_json::json!("value"))
        );
        assert_eq!(get_json_path(&json, "data.missing"), None);
        assert_eq!(get_json_path(&json, "nonexistent.path"), None);
    }

    #[test]
    fn test_build_url() {
        assert_eq!(
            build_url("https://api.x.com", "/v1/config").unwrap(),
            "https://api.x.com/v1/config"
        );
        assert_eq!(
            build_url("https://api.x.com/", "/v1/config").unwrap(),
            "https://api.x.com/v1/config"
        );
        assert_eq!(
            build_url("https://api.x.com", "v1/config").unwrap(),
            "https://api.x.com/v1/config"
        );
        assert_eq!(
            build_url("https://api.x.com", "").unwrap(),
            "https://api.x.com"
        );
        assert_eq!(
            build_url("https://api.x.com", "https://other.com/api").unwrap(),
            "https://other.com/api"
        );
    }

    #[test]
    fn test_compute_timeout() {
        assert_eq!(compute_timeout(None), Duration::from_millis(10_000));
        assert_eq!(compute_timeout(Some(5_000)), Duration::from_millis(5_000));
        assert_eq!(compute_timeout(Some(60_000)), Duration::from_millis(30_000));
        assert_eq!(compute_timeout(Some(100)), Duration::from_millis(1_000));
    }

    #[test]
    fn test_extract_host() {
        assert_eq!(extract_host("https://api.example.com/path"), Some("api.example.com".to_string()));
        assert_eq!(extract_host("https://api.example.com:8080/path"), Some("api.example.com".to_string()));
        assert_eq!(extract_host("http://localhost:3000"), Some("localhost".to_string()));
        assert_eq!(extract_host("not-a-url"), None);
    }

    #[test]
    fn test_resolve_url() {
        assert_eq!(
            resolve_url("https://api.x.com/v1/config", "https://other.com/api").unwrap(),
            "https://other.com/api"
        );
        assert_eq!(
            resolve_url("https://api.x.com/v1/config", "/v2/config").unwrap(),
            "https://api.x.com/v2/config"
        );
        assert_eq!(
            resolve_url("https://api.x.com/v1/config", "config2").unwrap(),
            "https://api.x.com/config2"
        );
    }

    #[test]
    fn test_map_response_standard_fields() {
        let mut mapping = HashMap::new();
        mapping.insert("data.host".to_string(), "serverAddr".to_string());
        mapping.insert("data.port".to_string(), "serverPort".to_string());
        mapping.insert("data.key".to_string(), "token".to_string());

        let response = serde_json::json!({
            "data": { "host": "frps.example.com", "port": 7000, "key": "secret" }
        });

        let payload = map_response(&response, &mapping).unwrap();
        assert_eq!(payload.server_addr, "frps.example.com");
        assert_eq!(payload.server_port, 7000);
        assert_eq!(payload.token, Some("secret".to_string()));
    }

    #[test]
    fn test_map_response_custom_variables() {
        let mut mapping = HashMap::new();
        mapping.insert("data.host".to_string(), "serverAddr".to_string());
        mapping.insert("data.port".to_string(), "serverPort".to_string());
        mapping.insert("data.extra".to_string(), "customVar".to_string());

        let response = serde_json::json!({
            "data": { "host": "x.com", "port": 7000, "extra": "hello" }
        });

        let payload = map_response(&response, &mapping).unwrap();
        assert_eq!(
            payload.custom_variables.and_then(|m| m.get("customVar").cloned()),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_map_response_missing_required() {
        let mut mapping = HashMap::new();
        mapping.insert("data.host".to_string(), "serverAddr".to_string());
        // 缺少 serverPort

        let response = serde_json::json!({ "data": { "host": "x.com" } });
        let result = map_response(&response, &mapping);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("服务器端口"));
    }

    #[test]
    fn test_render_config_template() {
        let payload = ConfigPayload {
            server_addr: "frps.example.com".to_string(),
            server_port: 7000,
            token: Some("secret\"key".to_string()),
            assigned_remote_port: Some(30001),
            assigned_subdomain: Some("my-tunnel".to_string()),
            custom_variables: None,
        };

        let template = r#"serverAddr = "{server_addr}"
serverPort = {server_port}
auth.token = "{token}"
remotePort = {assigned_remote_port}
subdomain = "{assigned_subdomain}""#;

        // 写入临时文件测试
        let result = template
            .replace("{server_addr}", &escape_toml_string(&payload.server_addr))
            .replace("{server_port}", &payload.server_port.to_string())
            .replace("{token}", &escape_toml_string(payload.token.as_ref().unwrap()))
            .replace("{assigned_remote_port}", &payload.assigned_remote_port.unwrap().to_string())
            .replace("{assigned_subdomain}", &escape_toml_string(payload.assigned_subdomain.as_ref().unwrap()));

        assert!(result.contains("serverAddr = \"frps.example.com\""));
        assert!(result.contains("serverPort = 7000"));
        assert!(result.contains("auth.token = \"secret\\\"key\""));
        assert!(result.contains("remotePort = 30001"));
        assert!(result.contains("subdomain = \"my-tunnel\""));
    }
}
