//! OAuth Token 获取
//!
//! 支持两种流程：
//! - **Web Auth Code Flow**（官方 ID）：浏览器授权 → 回调获取 code → 换取 token
//! - **Device Code Flow**（自定义 ID）：设备码 → 用户浏览器输入 → 轮询获取 token

use crate::http;
use crate::log_info;
use crate::log_warn;
use super::config;
use super::types::*;

/// 构建 Web Auth Code Flow 的授权 URL
pub fn build_auth_url() -> String {
    let ep = config::endpoints();
    // scope 中的空格需要编码为 %20，redirect_uri 无需编码（Microsoft OAuth 兼容）
    let scope = ep.scope.replace(' ', "%20");
    format!(
        "{}?client_id={}&response_type=code&scope={}&redirect_uri={}",
        ep.authorize_url,
        config::OAUTH_CLIENT_ID,
        scope,
        ep.redirect_uri,
    )
}

/// Web Auth Code Flow：用授权码换取 OAuth Token
pub async fn exchange_auth_code(code: &str) -> Result<OAuthTokenResponse, MicrosoftLoginError> {
    let ep = config::endpoints();
    let client = http::get_client();

    let params = [
        ("client_id", config::OAUTH_CLIENT_ID),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", ep.redirect_uri),
        ("scope", ep.scope),
    ];

    let response = client
        .post(ep.token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| MicrosoftLoginError::new(format!("auth_code request error: {}", e))
            .with_step("auth_code"))?;

    let body_text = response.text().await.unwrap_or_default();

    if let Ok(body) = serde_json::from_str::<serde_json::Value>(&body_text) {
        if let Some(error) = body.get("error").and_then(|v| v.as_str()) {
            let desc = body.get("error_description").and_then(|v| v.as_str()).unwrap_or("");
            return Err(MicrosoftLoginError::new(format!("auth_code error: {} - {}", error, desc))
                .with_step("auth_code")
                .with_code(error));
        }
    }

    let token: OAuthTokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| MicrosoftLoginError::new(format!("auth_code parse error: {}", e))
            .with_step("auth_code"))?;

    log_info!("OAuth token obtained via auth code");
    Ok(token)
}

/// Device Code Flow：申请设备码（仅自定义 ID 可用）
pub async fn request_device_code() -> Result<DeviceCodeResponse, MicrosoftLoginError> {
    let ep = config::endpoints();
    let device_code_url = ep.authorize_url.replace("authorize", "devicecode");
    log_info!("Requesting device code for Microsoft login");

    let params = [
        ("client_id", config::OAUTH_CLIENT_ID),
        ("scope", ep.scope),
    ];

    let response = http::get_client()
        .post(&device_code_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| MicrosoftLoginError::new(format!("device_code request error: {}", e))
            .with_step("device_code"))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        log_warn!("Device code request failed: {} - {}", status, body_text);
        return Err(MicrosoftLoginError::new(format!("device_code HTTP {}: {}", status, body_text))
            .with_step("device_code"));
    }

    let result: DeviceCodeResponse = serde_json::from_str(&body_text)
        .map_err(|e| MicrosoftLoginError::new(format!("device_code parse error: {}", e))
            .with_step("device_code"))?;

    log_info!("Device code obtained: user_code={}, verification_uri={}",
        result.user_code, result.verification_uri);
    Ok(result)
}

/// Device Code Flow：轮询授权结果
///
/// `Ok(Some(token))` = 授权成功；`Ok(None)` = 继续轮询；`Err` = 失败。
pub async fn poll_device_code(device_code: &str) -> Result<Option<OAuthTokenResponse>, MicrosoftLoginError> {
    let ep = config::endpoints();
    let client = http::get_client();

    let params = [
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ("client_id", config::OAUTH_CLIENT_ID),
        ("device_code", device_code),
        ("scope", ep.scope),
    ];

    let response = client
        .post(ep.token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| MicrosoftLoginError::new(format!("poll request error: {}", e)).with_step("poll"))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();

    if let Some(error) = body.get("error").and_then(|v| v.as_str()) {
        return match error {
            "authorization_pending" | "slow_down" => Ok(None),
            "authorization_declined" => Err(MicrosoftLoginError::new(format!("poll declined: {}", body_text))
                .with_step("poll").with_code(error)),
            "expired_token" => Err(MicrosoftLoginError::new(format!("poll expired: {}", body_text))
                .with_step("poll").with_code(error)),
            _ => Err(MicrosoftLoginError::new(format!("poll error: {}", body_text))
                .with_step("poll").with_code(error)),
        };
    }

    if !status.is_success() {
        return Err(MicrosoftLoginError::new(format!("poll HTTP {}: {}", status, body_text))
            .with_step("poll"));
    }

    let token: OAuthTokenResponse = serde_json::from_value(body)
        .map_err(|e| MicrosoftLoginError::new(format!("poll parse error: {}", e)).with_step("poll"))?;

    log_info!("OAuth token obtained successfully");
    Ok(Some(token))
}

/// 使用 Refresh Token 刷新 OAuth Token
pub async fn refresh_oauth_token(refresh_token: &str) -> Result<OAuthTokenResponse, MicrosoftLoginError> {
    let ep = config::endpoints();
    log_info!("Refreshing OAuth token");

    let params = [
        ("client_id", config::OAUTH_CLIENT_ID),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
        ("scope", ep.scope),
    ];

    let response = http::get_client()
        .post(ep.refresh_url)
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("X-Requested-With", "XMLHttpRequest")
        .form(&params)
        .send()
        .await
        .map_err(|e| MicrosoftLoginError::new(format!("refresh request error: {}", e)).with_step("refresh"))?;

    let body_text = response.text().await.unwrap_or_default();

    if let Ok(body) = serde_json::from_str::<serde_json::Value>(&body_text) {
        if let Some(error) = body.get("error").and_then(|v| v.as_str()) {
            let desc = body.get("error_description").and_then(|v| v.as_str()).unwrap_or("");
            return Err(MicrosoftLoginError::new(format!("refresh error: {} - {}", error, desc))
                .with_step("refresh").with_code(error));
        }
    }

    let token: OAuthTokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| MicrosoftLoginError::new(format!("refresh parse error: {}", e)).with_step("refresh"))?;

    log_info!("OAuth token refreshed successfully");
    Ok(token)
}
