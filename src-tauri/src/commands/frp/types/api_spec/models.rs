//! Open API 接口规范模型（endpoints.json 反序列化结构）
//!
//! 设计参考：docs/Frp Test/frp/api/endpoints.json。
//! 厂商接口响应结构各不相同，通过此规范将差异全部做成可配置项。

use serde::Deserialize;
use std::collections::HashMap;

use super::field_mapping::FieldMapping;

/// serde 默认值：配置格式 ini
fn default_config_format() -> String {
    "ini".to_string()
}

/// serde 默认值：bearer
fn default_auth_type_bearer() -> String {
    "bearer".to_string()
}

/// endpoints.json 顶层结构
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSpec {
    /// 规范版本
    #[serde(default)]
    pub spec_version: Option<String>,
    /// API 基础 URL（如 "https://api.openfrp.net"）
    pub base_url: String,
    /// token 注入配置
    #[serde(default)]
    pub auth: Option<AuthHeader>,
    /// 认证流程定义（请求参数 + 响应解析）
    #[serde(default)]
    pub auth_flows: Option<AuthFlows>,
    /// 全局响应包裹解析（各接口可用 endpoints.*.envelope 覆盖）
    #[serde(default)]
    pub envelope: Option<Envelope>,
    /// 配置获取模式
    pub config: ConfigMode,
    /// API 端点定义
    #[serde(default)]
    pub endpoints: Option<EndpointsDef>,
}

/// token 注入配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthHeader {
    /// 请求头字段名（如 "Authorization"）
    pub header_name: String,
    /// 值前缀（如 "Bearer "，无空格时留空）
    pub header_prefix: String,
    /// 登录响应中服务器密钥所在响应头名（解密用，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_key_name: Option<String>,
}

/// 认证流程定义
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthFlows {
    /// OAuth2 授权码流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<OAuth2Flow>,
    /// Device Code 设备码流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<DeviceCodeFlow>,
    /// API Key 直传流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyFlow>,
    /// 远程登录流程（如 OpenFRP argoAccess）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_login: Option<RemoteLoginFlow>,
}

/// OAuth2 流程（token 交换 + refresh）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Flow {
    pub token: FlowRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<FlowRequest>,
}

/// Device Code 流程（request + poll）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeFlow {
    pub request: FlowRequest,
    pub poll: FlowRequest,
}

/// API Key 流程（请求头/查询直传）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyFlow {
    pub header_name: String,
    /// 值模板（如 "OPENFRP{apiKey}"）
    pub header_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_name: Option<String>,
}

/// 远程登录流程（request + poll）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteLoginFlow {
    pub request: FlowRequest,
    pub poll: FlowRequest,
}

/// 流程请求定义（通用：method + url + body + response 解析）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: serde_json::Value,
    /// 响应字段提取规则
    #[serde(default)]
    pub response: HashMap<String, FieldExtractor>,
    /// Device Code 轮询时的 pending 错误标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_error: Option<String>,
}

/// 字段提取器（从 body 或 header 按 path/name 取值）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldExtractor {
    /// 来源：body / header
    pub from: String,
    /// from=body 时为 JSONPath（如 "$.access_token"）；from=header 时为头字段名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 响应包裹解析
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    /// 成功判断字段路径（JSONPath，如 "$.flag"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_field: Option<String>,
    /// 该字段等于此值时视为成功
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_value: Option<serde_json::Value>,
    /// 失败时错误消息字段路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_field: Option<String>,
    /// 数据字段路径（接口未覆盖时兜底）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_field: Option<String>,
}

/// 配置获取模式
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMode {
    /// url=厂商接口直返配置 / fields=启动器按字段拼装 / args=frpc 以启动参数运行
    pub mode: String,
    /// 配置格式（ini/toml），默认 ini
    #[serde(default = "default_config_format")]
    pub format: String,
    /// mode=args 时的参数模板（如 ["-u", "{token}", "-p", "{ids}"]）
    #[serde(default)]
    pub args: Vec<String>,
}

/// API 端点定义
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EndpointsDef {
    /// 账号信息端点（可选，验证授权有效性）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<EndpointDef>,
    /// 隧道端点集合
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnels: Option<TunnelsDef>,
}

/// 单个端点定义
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointDef {
    pub method: String,
    pub path: String,
    #[serde(default = "default_auth_type_bearer")]
    pub auth_type: String,
    #[serde(default)]
    pub query: HashMap<String, String>,
    /// 路径参数映射（如 { "tunnelId": "id" } 表示 {tunnelId} 替换为隧道 id 字段）
    #[serde(default)]
    pub path_params: HashMap<String, String>,
    /// 接口级 envelope 覆盖
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Envelope>,
    /// 响应映射
    pub response: ResponseDef,
}

/// 隧道端点集合
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TunnelsDef {
    /// 隧道列表端点（必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<EndpointDef>,
    /// 隧道详情端点（可选，列表未返回完整字段时启用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<EndpointDef>,
    /// 配置获取端点（可选，config.mode=url 时使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<EndpointDef>,
}

/// 响应映射定义
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseDef {
    /// 列表位置（仅 tunnels.list，如 "$.data[*].proxies[*]"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_field: Option<String>,
    /// 隧道 ID 字段名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_id_field: Option<String>,
    /// 隧道名称字段名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_name_field: Option<String>,
    /// 数据字段路径（如 "$.data"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_field: Option<String>,
    /// 字段映射（key=统一字段名，value=厂商字段名或 {field, split} 对象）
    #[serde(default)]
    pub fields: HashMap<String, FieldMapping>,
    /// 响应编码（text/json），config 端点用 text 取原始字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}