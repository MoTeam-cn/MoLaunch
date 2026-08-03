//! 认证返回类型（AuthStatus / OAuth2Result / DeviceCodeResult / DeviceCodePollResult）

use serde::Serialize;

/// 认证状态（get_auth_status 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub provider_id: String,
    /// 是否已认证（有有效 token）
    pub authenticated: bool,
    /// 认证类型：none / oauth2 / device_code / api_key
    pub auth_type: String,
    /// token 过期时间（Unix 秒），已过期时仍返回供前端展示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    /// 续期中：token 已过期但存在 refresh_token，正在静默续期
    #[serde(default, skip_serializing_if = "is_false")]
    pub refreshing: bool,
}

/// serde 辅助：bool 默认值
fn is_false(b: &bool) -> bool {
    !*b
}

/// OAuth2 流程结果（start_oauth2 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Result {
    /// token 过期时间（Unix 秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// Device Code 流程启动结果（start_device_code 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResult {
    /// 用户码（前端显示给用户输入）
    pub user_code: String,
    /// 验证链接（用户访问此 URL 输入用户码）
    pub verification_uri: String,
    /// 过期时间（秒）
    pub expires_in: u64,
    /// 轮询间隔（秒）
    pub interval: u64,
}

/// Device Code 轮询结果（poll_device_code 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodePollResult {
    /// 状态：pending / success / expired / declined / slow_down
    pub status: String,
    /// token 过期时间（仅 status=success 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围（仅 status=success 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}
