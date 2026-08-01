//! Frp 厂商认证模块：OAuth2 / Device Code / API Key 三种流程
//!
//! token 使用 OS 密钥存储（Windows Credential Manager / macOS Keychain / Linux Secret Service）。
//! 子模块：storage（密钥存储辅助）/ oauth2 / device_code / api_key。

use super::provider::{read_provider_manifest, SYSTEM_DEFAULT_ID};
use crate::log_info;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

mod api_key;
mod device_code;
mod oauth2;
mod storage;

// ============================================================
// 返回类型
// ============================================================

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

// ============================================================
// 内部共享类型
// ============================================================

/// OAuth2 / Device Code token 端点响应
#[derive(Debug, Deserialize)]
pub(super) struct TokenResponse {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
    /// 错误字段（Device Code 轮询时使用）
    pub error: Option<String>,
    #[allow(dead_code)]
    pub error_description: Option<String>,
}

// ============================================================
// 公共函数
// ============================================================

/// 查询指定厂商的认证状态
///
/// - auth_type=none：始终 authenticated=true
/// - auth_type=oauth2/device_code：检查 access_token 是否存在且未过期
/// - auth_type=api_key：检查 access_token（即 API Key）是否存在
///
/// expires_at 即使已过期也会返回，前端据此区分「即将过期」/「已过期」。
pub async fn get_auth_status(provider_id: &str) -> Result<AuthStatus, String> {
    // 系统默认厂商无需认证
    if provider_id == SYSTEM_DEFAULT_ID {
        return Ok(AuthStatus {
            provider_id: provider_id.to_string(),
            authenticated: true,
            auth_type: "none".to_string(),
            expires_at: None,
            scopes: None,
        });
    }

    let manifest = read_provider_manifest(provider_id)?;
    let auth_type = manifest.auth.auth_type.clone();

    if auth_type == "none" {
        return Ok(AuthStatus {
            provider_id: provider_id.to_string(),
            authenticated: true,
            auth_type,
            expires_at: None,
            scopes: None,
        });
    }

    // 检查 access_token 是否存在
    let access_token = storage::load_secret(provider_id, storage::KEY_ACCESS_TOKEN)?;
    let authenticated = access_token.is_some();

    // 检查是否过期（仅 oauth2 / device_code 有过期时间）
    let expires_at = if matches!(auth_type.as_str(), "oauth2" | "device_code") {
        storage::load_expires_at(provider_id)?
    } else {
        None
    };

    // token 存在但已过期 -> authenticated=false
    let authenticated = if authenticated {
        match expires_at {
            Some(exp) => exp > storage::now_secs(),
            None => true, // api_key 无过期时间
        }
    } else {
        false
    };

    let scopes = storage::load_scopes(provider_id)?;

    Ok(AuthStatus {
        provider_id: provider_id.to_string(),
        authenticated,
        auth_type,
        expires_at,
        scopes,
    })
}

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

/// 刷新 token
///
/// access_token 过期前 5 分钟用 refresh_token 刷新。也可由用户手动触发。
pub async fn refresh_token(_state: &AppState, provider_id: &str) -> Result<(), String> {
    let manifest = read_provider_manifest(provider_id)?;

    // 获取 tokenUrl + clientId（oauth2 或 device_code）
    let (token_url, client_id) = if let Some(ref oauth2) = manifest.auth.oauth2 {
        (oauth2.token_url.clone(), oauth2.client_id.clone())
    } else if let Some(ref dc) = manifest.auth.device_code {
        (dc.token_url.clone(), dc.client_id.clone())
    } else {
        return Err(format!("厂商 {} 不支持 token 刷新", provider_id));
    };

    let refresh_token = storage::load_secret(provider_id, storage::KEY_REFRESH_TOKEN)?
        .ok_or_else(|| format!("厂商 {} 无 refresh_token，请重新认证", provider_id))?;

    log_info!("[Frp Auth] 刷新 token: provider={}", provider_id);

    let client = crate::http::get_client();
    let resp = client
        .post(&token_url)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("client_id", client_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("刷新 token 请求失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        crate::log_error!("[Frp Auth] 刷新 token 失败: HTTP {} {}", status, body);
        return Err(format!("刷新 token 失败: HTTP {}", status));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析刷新响应失败: {}", e))?;

    let access_token = token_resp.access_token.ok_or("刷新响应缺少 access_token")?;

    storage::store_token_info(
        provider_id,
        &access_token,
        token_resp.refresh_token.as_deref(),
        token_resp.expires_in,
        None, // 刷新不改变 scopes
    )?;

    log_info!("[Frp Auth] token 刷新成功: provider={}", provider_id);
    Ok(())
}

/// 撤销认证（删除所有存储的 token）
pub async fn revoke_auth(provider_id: &str) -> Result<(), String> {
    log_info!("[Frp Auth] 撤销认证: provider={}", provider_id);

    // 清除 keyring 中的所有密钥
    storage::delete_secret(provider_id, storage::KEY_ACCESS_TOKEN)?;
    storage::delete_secret(provider_id, storage::KEY_REFRESH_TOKEN)?;
    storage::delete_secret(provider_id, storage::KEY_EXPIRES_AT)?;
    storage::delete_secret(provider_id, storage::KEY_SCOPES)?;

    // 清除 Device Code 会话
    device_code::remove_device_code_session(provider_id);

    Ok(())
}

/// 读取 access_token（供 api_schema 模块调用厂商 API 时使用）
///
/// 仅读取已存储的 access_token，不检查过期、不自动刷新。
/// 调用方（api_schema::fetch_vendor_config）应先调用 refresh_token 确保有效。
pub async fn load_token(provider_id: &str) -> Result<String, String> {
    storage::load_secret(provider_id, storage::KEY_ACCESS_TOKEN)?
        .ok_or_else(|| format!("厂商 {} 未认证，请先完成认证", provider_id))
}

/// 保存 API Key（auth_type=api_key 时由前端调用）
///
/// API Key 直接作为 access_token 存储，无过期时间、无 refresh_token。
pub async fn save_api_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    api_key::save_api_key(provider_id, api_key).await
}
