//! 认证配置与 auth.json 交互层类型

use serde::{Deserialize, Serialize};

/// serde 默认值：none
fn default_auth_type() -> String {
    "none".to_string()
}

/// serde 默认值：Device Code 轮询间隔 5 秒
fn default_poll_interval() -> u64 {
    5
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    /// 认证类型：none / oauth2 / device_code / api_key
    #[serde(default = "default_auth_type")]
    #[serde(rename = "type")]
    pub auth_type: String,
    /// OAuth2 配置（type=oauth2 时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<OAuth2Config>,
    /// Device Code 配置（type=device_code 时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<DeviceCodeConfig>,
    /// API Key 配置（type=api_key 时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyConfig>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            auth_type: default_auth_type(),
            oauth2: None,
            device_code: None,
            api_key: None,
        }
    }
}

/// OAuth2 配置（auth.type=oauth2 时必填）
///
/// 参见 FRP_MANAGER_DESIGN.md §6.3。本地启动 HTTP 服务监听 redirectPort 接收回调，
/// 浏览器跳转走 `crate::minecraft::system::shell::open_url`，token 交换在后端完成。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Config {
    /// 授权页 URL
    pub authorize_url: String,
    /// token 交换 URL（兼容旧版 manifest，新设计改由 endpoints.json authFlows.oauth2.token.url 提供）
    pub token_url: String,
    /// 客户端 ID
    pub client_id: String,
    /// 客户端密钥（可选，部分厂商需要）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// 权限范围
    #[serde(default)]
    pub scopes: Vec<String>,
    /// 回调端口（本地启动 HTTP 服务接收 callback）
    pub redirect_port: u16,
}

/// Device Code 配置（auth.type=device_code 时必填）
///
/// 参见 FRP_MANAGER_DESIGN.md §6.4。POST deviceCodeUrl 获取设备码，
/// 前端显示用户码 + 验证链接 + 倒计时，后端按 interval 轮询 tokenUrl。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeConfig {
    /// 设备码请求 URL
    pub device_code_url: String,
    /// token 轮询 URL
    pub token_url: String,
    /// 客户端 ID
    pub client_id: String,
    /// 客户端密钥（可选，部分厂商需要）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// 权限范围
    #[serde(default)]
    pub scopes: Vec<String>,
    /// 轮询间隔（秒），默认 5
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,
}

/// API Key 配置（auth.type=api_key 时必填）
///
/// 用户手动获取 Key 填入，存储到 OS 密钥存储，调用厂商 API 时注入请求头。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyConfig {
    /// 获取 API Key 的 URL（前端提供跳转入口）
    pub obtain_url: String,
    /// API Key 在请求头中的字段名
    pub header_name: String,
}

// auth.json 认证交互层类型
/// auth.json 结构（认证交互层配置）
///
/// 仅描述用户交互方式（授权页 URL、回调端口等），
/// 实际网络请求与响应解析见 endpoints.json 的 authFlows。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFile {
    /// 认证类型：none / oauth2 / device_code / api_key
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<AuthFileOAuth2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<AuthFileDeviceCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<AuthFileApiKey>,
}

/// auth.json 中的 OAuth2 交互配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFileOAuth2 {
    pub authorize_url: String,
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub redirect_port: u16,
}

/// auth.json 中的 Device Code 交互配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFileDeviceCode {
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,
}

/// auth.json 中的 API Key 交互配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFileApiKey {
    pub obtain_url: String,
    pub header_name: String,
}
