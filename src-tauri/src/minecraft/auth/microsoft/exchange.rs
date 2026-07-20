//! Token 交换链
//!
//! OAuth Token → XBL → XSTS → MC Token → 验证所有权 → 获取档案

use crate::http;
use crate::log_info;
use crate::log_warn;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use super::types::*;

/// XBL 认证端点
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
/// XSTS 认证端点
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
/// Minecraft 认证端点
const MC_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
/// Minecraft 游戏所有权验证端点
const MC_ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
/// Minecraft 玩家档案端点
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

/// 步骤 2：OAuth Token → XBL Token
async fn exchange_xbl_token(oauth_token: &str) -> Result<XblTokenResponse, MicrosoftLoginError> {
    log_info!("Exchanging OAuth token for XBL token");
    // v2.0 端点获取的 token 需要 `d=` 前缀；旧版 login.live.com 端点不需要
    let rps_ticket = if super::config::use_v2_endpoints() {
        format!("d={}", oauth_token)
    } else {
        oauth_token.to_string()
    };
    let body = serde_json::json!({
        "Properties": { "AuthMethod": "RPS", "SiteName": "user.auth.xboxlive.com", "RpsTicket": rps_ticket },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });
    let resp = http::get_client().post(XBL_AUTH_URL)
        .header("Content-Type", "application/json").header("Accept", "application/json")
        .json(&body).send().await
        .map_err(|e| MicrosoftLoginError::new(format!("xbl request error: {}", e)).with_step("xbl"))?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(MicrosoftLoginError::new(format!("xbl HTTP {}: {}", status, body_text)).with_step("xbl"));
    }
    let result: XblTokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| MicrosoftLoginError::new(format!("xbl parse error: {}", e)).with_step("xbl"))?;
    log_info!("XBL token obtained successfully");
    Ok(result)
}

/// 步骤 3：XBL Token → XSTS Token + UHS
async fn exchange_xsts_token(xbl_token: &str) -> Result<(String, String), MicrosoftLoginError> {
    log_info!("Exchanging XBL token for XSTS token");
    let body = serde_json::json!({
        "Properties": { "SandboxId": "RETAIL", "UserTokens": [xbl_token] },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });
    let resp = http::get_client().post(XSTS_AUTH_URL)
        .header("Content-Type", "application/json").header("Accept", "application/json")
        .json(&body).send().await
        .map_err(|e| MicrosoftLoginError::new(format!("xsts request error: {}", e)).with_step("xsts"))?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        let lower = body_text.to_lowercase();
        let code = if lower.contains("2148916227") { Some("2148916227") }
            else if lower.contains("2148916233") { Some("2148916233") }
            else if lower.contains("2148916235") { Some("2148916235") }
            else if lower.contains("2148916238") { Some("2148916238") }
            else { None };
        let mut err = MicrosoftLoginError::new(format!("xsts HTTP {}: {}", status, body_text)).with_step("xsts");
        if let Some(c) = code { err = err.with_code(c); }
        return Err(err);
    }
    let result: XstsTokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| MicrosoftLoginError::new(format!("xsts parse error: {}", e)).with_step("xsts"))?;
    let uhs = result.display_claims.as_ref()
        .and_then(|dc| dc.get("xui")).and_then(|xui| xui.get(0))
        .and_then(|item| item.get("uhs")).and_then(|v| v.as_str())
        .ok_or_else(|| MicrosoftLoginError::new(format!("xsts missing UHS: {}", body_text)).with_step("xsts"))?
        .to_string();
    log_info!("XSTS token and UHS obtained successfully");
    Ok((result.token, uhs))
}

/// 步骤 4：XSTS Token + UHS → Minecraft Access Token
async fn exchange_mc_token(xsts_token: &str, uhs: &str) -> Result<MinecraftLoginResponse, MicrosoftLoginError> {
    log_info!("Exchanging XSTS token for Minecraft token");
    let identity_token = format!("XBL3.0 x={};{}", uhs, xsts_token);
    let body = serde_json::json!({ "identityToken": identity_token });
    let resp = http::get_client().post(MC_AUTH_URL)
        .header("Content-Type", "application/json").header("Accept", "application/json")
        .json(&body).send().await
        .map_err(|e| MicrosoftLoginError::new(format!("mc_token request error: {}", e)).with_step("mc_token"))?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(MicrosoftLoginError::new(format!("mc_token HTTP {}: {}", status, body_text))
            .with_step("mc_token").with_code(status.as_u16().to_string()));
    }
    let result: MinecraftLoginResponse = serde_json::from_str(&body_text)
        .map_err(|e| MicrosoftLoginError::new(format!("mc_token parse error: {}", e)).with_step("mc_token"))?;
    log_info!("Minecraft token obtained successfully");
    Ok(result)
}

/// 步骤 5：验证游戏所有权
async fn check_entitlements(mc_token: &str) -> Result<bool, MicrosoftLoginError> {
    log_info!("Checking game entitlements");
    let resp = http::get_client().get(MC_ENTITLEMENTS_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send().await
        .map_err(|e| MicrosoftLoginError::new(format!("entitlements request error: {}", e)).with_step("entitlements"))?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        log_warn!("Entitlements check failed: {} - {}", status, body_text);
        return Ok(false);
    }
    let body: serde_json::Value = serde_json::from_str(&body_text)
        .map_err(|e| MicrosoftLoginError::new(format!("entitlements parse error: {}", e)).with_step("entitlements"))?;
    let has_game = body.get("items").and_then(|i| i.as_array()).map(|i| !i.is_empty()).unwrap_or(false);
    if !has_game {
        return Err(MicrosoftLoginError::new(format!("entitlements: no game ownership. response: {}", body_text))
            .with_step("entitlements"));
    }
    log_info!("Game ownership verified");
    Ok(true)
}

/// 步骤 6：获取玩家档案
async fn fetch_profile(mc_token: &str) -> Result<MinecraftProfile, MicrosoftLoginError> {
    log_info!("Fetching Minecraft profile");
    let resp = http::get_client().get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send().await
        .map_err(|e| MicrosoftLoginError::new(format!("profile request error: {}", e)).with_step("profile"))?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(MicrosoftLoginError::new(format!("profile HTTP {}: {}", status, body_text))
            .with_step("profile").with_code(status.as_u16().to_string()));
    }
    let profile: MinecraftProfile = serde_json::from_str(&body_text)
        .map_err(|e| MicrosoftLoginError::new(format!("profile parse error: {}", e)).with_step("profile"))?;
    log_info!("Profile obtained: username={}, uuid={}", profile.name, profile.id);
    Ok(profile)
}

/// 完成从 OAuth Token 开始的完整交换链
///
/// `progress` 回调在每个步骤开始时被调用。
pub async fn complete_login_chain<F>(
    oauth_access_token: &str,
    oauth_refresh_token: &str,
    mut progress: F,
) -> Result<MicrosoftLoginResult, MicrosoftLoginError>
where F: FnMut(&str) {
    progress("xbl");
    let xbl = exchange_xbl_token(oauth_access_token).await?;
    progress("xsts");
    let (xsts, uhs) = exchange_xsts_token(&xbl.token).await?;
    progress("mc_token");
    let mc = exchange_mc_token(&xsts, &uhs).await?;
    progress("entitlements");
    check_entitlements(&mc.access_token).await?;
    progress("profile");
    let profile = fetch_profile(&mc.access_token).await?;

    let expires_at = unix_now() + mc.expires_in.saturating_sub(1200);
    let profile_json = serde_json::to_string(&profile).unwrap_or_default();

    Ok(MicrosoftLoginResult {
        username: profile.name,
        uuid: profile.id,
        access_token: mc.access_token,
        refresh_token: oauth_refresh_token.to_string(),
        expires_at,
        profile_json,
    })
}

/// 检查 Token 是否已过期
pub fn is_token_expired(expires_at: u64) -> bool {
    unix_now() >= expires_at
}

/// 获取轮询间隔
pub fn get_poll_interval(server_interval: u64) -> Duration {
    if server_interval > 1 { Duration::from_secs(server_interval - 1) }
    else { Duration::from_secs(2) }
}
