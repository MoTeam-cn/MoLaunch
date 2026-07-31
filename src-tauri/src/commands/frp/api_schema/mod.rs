//! 厂商 API 引擎：api-schema.json 解析 + API 调用 + 配置模板渲染
//!
//! 认证后调用厂商 API 拉取 frpc 配置，按 api-schema.json 动态构造 HTTP 请求，
//! 映射响应到 ConfigPayload，填充 config-template.toml 生成最终 frpc 配置。
//! 子模块：http（请求发送）/ mapping（响应映射）/ helpers（URL 与客户端辅助）。

use super::{providers_root, validate_provider_id};
use crate::log_info;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod helpers;
mod http;
mod mapping;
#[cfg(test)]
mod tests;

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
/// 流程：加载 schema → 加载 access_token → 获取 device_id →
/// 构造并发送 HTTP 请求（注入 token + 填充参数）→ 按 response_mapping 映射响应。
pub async fn fetch_vendor_config(
    state: &AppState,
    provider_id: &str,
) -> Result<ConfigPayload, String> {
    let schema = load_api_schema(provider_id)?;

    let token = crate::commands::frp::auth::load_token(provider_id)
        .await
        .map_err(|e| format!("加载厂商 {} 的 access_token 失败: {}", provider_id, e))?;

    let device_id = crate::commands::sdk::get_device_id(state)
        .await
        .map_err(|e| format!("获取 device_id 失败: {}", e))?;

    log_info!(
        "[Frp] 拉取厂商 {} 配置: {} {}",
        provider_id,
        schema.endpoints.fetch_config.method,
        schema.endpoints.fetch_config.path
    );

    let response = http::send_api_request(
        &schema,
        &schema.endpoints.fetch_config,
        &token,
        &device_id,
        provider_id,
    )
    .await?;

    let payload = mapping::map_response(&response, &schema.endpoints.fetch_config.response_mapping)?;

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

    result = result.replace(
        "{server_addr}",
        &helpers::escape_toml_string(&payload.server_addr),
    );
    result = result.replace("{server_port}", &payload.server_port.to_string());

    match &payload.token {
        Some(t) => result = result.replace("{token}", &helpers::escape_toml_string(t)),
        None => result = result.replace("{token}", ""),
    }

    match payload.assigned_remote_port {
        Some(p) => result = result.replace("{assigned_remote_port}", &p.to_string()),
        None => result = result.replace("{assigned_remote_port}", ""),
    }

    match &payload.assigned_subdomain {
        Some(s) => result = result.replace("{assigned_subdomain}", &helpers::escape_toml_string(s)),
        None => result = result.replace("{assigned_subdomain}", ""),
    }

    if let Some(ref vars) = payload.custom_variables {
        for (k, v) in vars {
            result = result.replace(&format!("{{{}}}", k), &helpers::escape_toml_string(v));
        }
    }

    Ok(result)
}
