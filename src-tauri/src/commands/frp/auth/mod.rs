//! Frp 厂商认证模块：OAuth2 / Device Code / API Key 三种流程
//!
//! token 经 SDK 内置 DES 加密后存文件（`<base_dir>/frp/auth/{provider_id}.json`）。
//! 子模块：storage（加密存储辅助）/ oauth2 / device_code / api_key / flows（可配置流程引擎）/
//! handlers（状态查询/撤销/token 注入等公开 API 处理函数）。

use super::types::{FieldExtractor, FlowRequest};
use crate::sdk::SdkInstance;
use crate::state::AppState;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

mod api_key;
mod device_code;
mod flows;
mod handlers;
mod oauth2;
mod storage;

/// 注入 SDK 引用（lib.rs 启动时调用，供 token 加密存储使用）
pub fn set_sdk(sdk: Arc<TokioMutex<Option<SdkInstance>>>) {
    storage::set_sdk(sdk);
}

// 返回类型
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

// 内部辅助（refresh_token 用 flows 引擎提取字段）
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
pub(super) fn extract_flow_error(resp: &flows::FlowResponse, flow: &FlowRequest) -> String {
    let err = resp.extract_field(get_extractor(flow, "errorField"));
    let desc = resp.extract_field(get_extractor(flow, "errorDescription"));
    match (err, desc) {
        (Some(e), Some(d)) if !e.is_empty() && !d.is_empty() => format!("{}: {}", e, d),
        (Some(e), _) if !e.is_empty() => e,
        (Some(e), _) => e,
        _ => "未知错误".to_string(),
    }
}

// 公开 API 入口（复杂处理逻辑在 handlers 子模块）
/// 启动 OAuth2 授权流程
///
/// 流程：启动本地 HTTP 服务监听 redirectPort → 打开浏览器跳转授权页 →
/// 等待回调 → 用 code 换取 token → 存储 token 到 OS 密钥存储。
pub async fn start_oauth2(state: &AppState, provider_id: &str) -> Result<OAuth2Result, String> {
    oauth2::start_oauth2(state, provider_id).await
}

/// 启动 Device Code 流程
///
/// 流程：POST deviceCodeUrl 获取设备码 → 返回用户码 + 验证链接 + 倒计时 →
/// 将 device_code 存入内存会话（供 poll_device_code 使用）。
pub async fn start_device_code(
    state: &AppState,
    provider_id: &str,
) -> Result<DeviceCodeResult, String> {
    device_code::start_device_code(state, provider_id).await
}

/// 轮询 Device Code token
///
/// 前端按 interval 调用，后端用 device_code 向 tokenUrl 发起请求：
/// pending → 继续轮询；success → 存储 token；expired/declined → 终止；slow_down → 增大间隔。
pub async fn poll_device_code(
    state: &AppState,
    provider_id: &str,
) -> Result<DeviceCodePollResult, String> {
    device_code::poll_device_code(state, provider_id).await
}

/// 保存 API Key（auth_type=api_key 时由前端调用）
///
/// API Key 直接作为 access_token 存储，无过期时间、无 refresh_token。
pub async fn save_api_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    api_key::save_api_key(provider_id, api_key).await
}

// 处理函数 re-export（调用方经 `crate::commands::frp::auth::xxx` 访问）
pub use handlers::{
    ensure_valid_token, get_auth_status, load_token, refresh_token, revoke_auth,
};
