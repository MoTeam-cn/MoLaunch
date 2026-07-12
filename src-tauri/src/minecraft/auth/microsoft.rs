//! 微软登录模块
//!
//! 实现 Device Code Flow 的完整 Token 交换链：
//! 1. 申请设备码 → 用户浏览器授权
//! 2. 轮询获取 OAuth Access Token + Refresh Token
//! 3. OAuth Token → XBL Token (user.auth.xboxlive.com)
//! 4. XBL Token → XSTS Token + UHS (xsts.auth.xboxlive.com)
//! 5. XSTS Token → Minecraft Access Token (api.minecraftservices.com)
//! 6. 验证游戏所有权 + 获取玩家档案
//!
//! 参考 PCL2 (Plain Craft Launcher 2) 的实现逻辑。

use crate::http;
use crate::log_info;
use crate::log_warn;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// OAuth Client ID（MoLaunch 注册的应用 ID）
/// 使用与 PCL2 / HMCL 等启动器一致的公共 Client ID
const OAUTH_CLIENT_ID: &str = "00000000402b5328";

/// OAuth Tenant
const OAUTH_TENANT: &str = "consumers";

/// OAuth Scope
const OAUTH_SCOPE: &str = "XboxLive.signin offline_access";

/// 设备码申请端点
const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";

/// Token 端点（设备码授权 + 刷新）
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

/// Token 刷新端点（login.live.com 兼容端点，与 PCL2 一致）
const REFRESH_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";

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

// ============================================================
// 响应类型
// ============================================================

/// 设备码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

/// OAuth Token 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: Option<String>,
}

/// XBL Token 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XblTokenResponse {
    pub issue_instant: String,
    pub not_after: String,
    pub token: String,
    pub display_claims: serde_json::Value,
}

/// XSTS Token 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XstsTokenResponse {
    pub issue_instant: String,
    pub not_after: String,
    pub token: String,
    pub display_claims: serde_json::Value,
}

/// Minecraft 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLoginResponse {
    pub username: String,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Minecraft 玩家档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub skins: Vec<serde_json::Value>,
    #[serde(default)]
    pub capes: Vec<serde_json::Value>,
}

/// 完整的微软登录结果（包含所有 Token，用于持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftLoginResult {
    /// 玩家用户名
    pub username: String,
    /// 玩家 UUID
    pub uuid: String,
    /// Minecraft Access Token（短期，约 24h）
    pub access_token: String,
    /// OAuth Refresh Token（长期，用于静默刷新）
    pub refresh_token: String,
    /// Access Token 过期时间戳（Unix 秒）
    pub expires_at: u64,
    /// 完整玩家档案 JSON
    pub profile_json: String,
}

// ============================================================
// 错误类型
// ============================================================

/// 微软登录错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftLoginError {
    pub message: String,
    pub error_code: Option<String>,
    pub step: Option<String>,
}

impl std::fmt::Display for MicrosoftLoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MicrosoftLoginError {}

impl MicrosoftLoginError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_code: None,
            step: None,
        }
    }

    fn with_step(mut self, step: impl Into<String>) -> Self {
        self.step = Some(step.into());
        self
    }

    fn with_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }
}

// ============================================================
// 核心逻辑
// ============================================================

/// 获取当前 Unix 时间戳（秒）
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 步骤 1a：申请设备码
pub async fn request_device_code() -> Result<DeviceCodeResponse, MicrosoftLoginError> {
    log_info!("Requesting device code for Microsoft login");

    let client = http::get_client();
    let params = [
        ("client_id", OAUTH_CLIENT_ID),
        ("tenant", OAUTH_TENANT),
        ("scope", OAUTH_SCOPE),
    ];

    let response = client
        .post(DEVICE_CODE_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            MicrosoftLoginError::new(format!("网络请求失败: {}", e)).with_step("device_code")
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log_warn!("Device code request failed: {} - {}", status, body);
        return Err(MicrosoftLoginError::new(format!(
            "申请设备码失败 (HTTP {}): {}",
            status, body
        ))
        .with_step("device_code"));
    }

    let result: DeviceCodeResponse = response.json().await.map_err(|e| {
        MicrosoftLoginError::new(format!("解析设备码响应失败: {}", e)).with_step("device_code")
    })?;

    log_info!(
        "Device code obtained: user_code={}, verification_uri={}",
        result.user_code,
        result.verification_uri
    );

    Ok(result)
}

/// 步骤 1b：轮询设备码授权结果
///
/// 返回 `Ok(Some(token))` 表示授权成功；
/// 返回 `Ok(None)` 表示需要继续轮询（authorization_pending）；
/// 返回 `Err` 表示失败或超时。
pub async fn poll_device_code(
    device_code: &str,
) -> Result<Option<OAuthTokenResponse>, MicrosoftLoginError> {
    let client = http::get_client();
    let params = [
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ("client_id", OAUTH_CLIENT_ID),
        ("device_code", device_code),
        ("scope", OAUTH_SCOPE),
    ];

    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| MicrosoftLoginError::new(format!("轮询请求失败: {}", e)).with_step("poll"))?;

    let status = response.status();
    let body: serde_json::Value = response.json().await.map_err(|e| {
        MicrosoftLoginError::new(format!("解析轮询响应失败: {}", e)).with_step("poll")
    })?;

    // 检查 error 字段
    if let Some(error) = body.get("error").and_then(|v| v.as_str()) {
        match error {
            "authorization_pending" => {
                return Ok(None); // 继续轮询
            }
            "authorization_declined" => {
                return Err(MicrosoftLoginError::new("用户拒绝了授权请求").with_step("poll"));
            }
            "expired_token" => {
                return Err(MicrosoftLoginError::new("设备码已过期，请重新登录").with_step("poll"));
            }
            "slow_down" => {
                // 需要降低轮询频率，返回 Ok(None) 由调用方增加间隔
                return Ok(None);
            }
            "invalid_grant" => {
                return Err(MicrosoftLoginError::new("授权无效，请重新登录").with_step("poll"));
            }
            _ => {
                let desc = body
                    .get("error_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                return Err(
                    MicrosoftLoginError::new(format!("授权失败: {} - {}", error, desc))
                        .with_step("poll")
                        .with_code(error),
                );
            }
        }
    }

    // HTTP 非 2xx 且没有 error 字段
    if !status.is_success() {
        return Err(
            MicrosoftLoginError::new(format!("轮询失败 (HTTP {})", status)).with_step("poll"),
        );
    }

    // 成功解析 Token
    let token: OAuthTokenResponse = serde_json::from_value(body).map_err(|e| {
        MicrosoftLoginError::new(format!("解析 Token 响应失败: {}", e)).with_step("poll")
    })?;

    log_info!("OAuth token obtained successfully");
    Ok(Some(token))
}

/// 步骤 1c：使用 Refresh Token 刷新 OAuth Token
pub async fn refresh_oauth_token(
    refresh_token: &str,
) -> Result<OAuthTokenResponse, MicrosoftLoginError> {
    log_info!("Refreshing OAuth token");

    let client = http::get_client();
    let params = [
        ("client_id", OAUTH_CLIENT_ID),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
        ("scope", OAUTH_SCOPE),
    ];

    let response = client
        .post(REFRESH_TOKEN_URL)
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("X-Requested-With", "XMLHttpRequest")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            MicrosoftLoginError::new(format!("刷新令牌请求失败: {}", e)).with_step("refresh")
        })?;

    let body_text = response.text().await.unwrap_or_default();

    // 检查是否需要重新登录
    let body_lower = body_text.to_lowercase();
    if body_lower.contains("must sign in again")
        || body_lower.contains("password expired")
        || body_lower.contains("is not valid")
        || body_lower.contains("expired")
    {
        return Err(
            MicrosoftLoginError::new("Refresh token 已失效，需要重新登录")
                .with_step("refresh")
                .with_code("Relogin"),
        );
    }

    // 尝试解析错误
    if let Ok(body) = serde_json::from_str::<serde_json::Value>(&body_text) {
        if let Some(error) = body.get("error").and_then(|v| v.as_str()) {
            if error == "invalid_grant" {
                return Err(
                    MicrosoftLoginError::new("Refresh token 已失效，需要重新登录")
                        .with_step("refresh")
                        .with_code("Relogin"),
                );
            }
            let desc = body
                .get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            return Err(
                MicrosoftLoginError::new(format!("刷新令牌失败: {} - {}", error, desc))
                    .with_step("refresh")
                    .with_code(error),
            );
        }
    }

    let token: OAuthTokenResponse = serde_json::from_str(&body_text).map_err(|e| {
        MicrosoftLoginError::new(format!("解析刷新响应失败: {}", e)).with_step("refresh")
    })?;

    log_info!("OAuth token refreshed successfully");
    Ok(token)
}

/// 步骤 2：OAuth Token → XBL Token
async fn exchange_xbl_token(oauth_token: &str) -> Result<XblTokenResponse, MicrosoftLoginError> {
    log_info!("Exchanging OAuth token for XBL token");

    let client = http::get_client();

    // PCL2: 若 AccessToken 已以 "d=" 开头则不加前缀
    let rps_ticket = if oauth_token.starts_with("d=") {
        oauth_token.to_string()
    } else {
        format!("d={}", oauth_token)
    };

    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": rps_ticket
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let response = client
        .post(XBL_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            MicrosoftLoginError::new(format!("XBL 认证请求失败: {}", e)).with_step("xbl")
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(
            MicrosoftLoginError::new(format!("XBL 认证失败 (HTTP {}): {}", status, body))
                .with_step("xbl"),
        );
    }

    let result: XblTokenResponse = response.json().await.map_err(|e| {
        MicrosoftLoginError::new(format!("解析 XBL 响应失败: {}", e)).with_step("xbl")
    })?;

    log_info!("XBL token obtained successfully");
    Ok(result)
}

/// 步骤 3：XBL Token → XSTS Token + UHS
async fn exchange_xsts_token(xbl_token: &str) -> Result<(String, String), MicrosoftLoginError> {
    log_info!("Exchanging XBL token for XSTS token");

    let client = http::get_client();

    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let response = client
        .post(XSTS_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            MicrosoftLoginError::new(format!("XSTS 认证请求失败: {}", e)).with_step("xsts")
        })?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        // 检查 Xbox 错误码（参考 PCL2 / PrismarineJS）
        let body_lower = body_text.to_lowercase();
        if body_lower.contains("2148916227") {
            return Err(MicrosoftLoginError::new("该账号已被 Xbox 封禁")
                .with_step("xsts")
                .with_code("2148916227"));
        }
        if body_lower.contains("2148916233") {
            return Err(MicrosoftLoginError::new(
                "该微软账号尚未注册 Xbox 账户，请先在 Xbox 官网注册",
            )
            .with_step("xsts")
            .with_code("2148916233"));
        }
        if body_lower.contains("2148916235") {
            return Err(MicrosoftLoginError::new("当前地区受限制，请尝试使用 VPN")
                .with_step("xsts")
                .with_code("2148916235"));
        }
        if body_lower.contains("2148916238") {
            return Err(
                MicrosoftLoginError::new("该账号年龄不足，需要将出生日期修改为 18 岁以上")
                    .with_step("xsts")
                    .with_code("2148916238"),
            );
        }

        return Err(MicrosoftLoginError::new(format!(
            "XSTS 认证失败 (HTTP {}): {}",
            status, body_text
        ))
        .with_step("xsts"));
    }

    let result: XstsTokenResponse = serde_json::from_str(&body_text).map_err(|e| {
        MicrosoftLoginError::new(format!("解析 XSTS 响应失败: {}", e)).with_step("xsts")
    })?;

    // 提取 UHS (UserHashString)
    let uhs = result
        .display_claims
        .get("xui")
        .and_then(|xui| xui.get(0))
        .and_then(|item| item.get("uhs"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| MicrosoftLoginError::new("XSTS 响应中缺少 UHS 字段").with_step("xsts"))?
        .to_string();

    log_info!("XSTS token and UHS obtained successfully");
    Ok((result.token, uhs))
}

/// 步骤 4：XSTS Token + UHS → Minecraft Access Token
async fn exchange_mc_token(
    xsts_token: &str,
    uhs: &str,
) -> Result<MinecraftLoginResponse, MicrosoftLoginError> {
    log_info!("Exchanging XSTS token for Minecraft token");

    let client = http::get_client();

    let identity_token = format!("XBL3.0 x={};{}", uhs, xsts_token);

    let body = serde_json::json!({
        "identityToken": identity_token
    });

    let response = client
        .post(MC_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            MicrosoftLoginError::new(format!("MC 认证请求失败: {}", e)).with_step("mc_token")
        })?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if status.as_u16() == 429 {
        return Err(MicrosoftLoginError::new("登录尝试太过频繁，请稍后再试")
            .with_step("mc_token")
            .with_code("429"));
    }
    if status.as_u16() == 403 {
        return Err(
            MicrosoftLoginError::new("当前 IP 登录尝试异常，请关闭 VPN 或更换节点后重试")
                .with_step("mc_token")
                .with_code("403"),
        );
    }

    // 检查账号封禁
    if body_text.contains("ACCOUNT_SUSPENDED") {
        return Err(MicrosoftLoginError::new("Minecraft 账号已被封禁")
            .with_step("mc_token")
            .with_code("ACCOUNT_SUSPENDED"));
    }

    if !status.is_success() {
        return Err(MicrosoftLoginError::new(format!(
            "MC 认证失败 (HTTP {}): {}",
            status, body_text
        ))
        .with_step("mc_token"));
    }

    let result: MinecraftLoginResponse = serde_json::from_str(&body_text).map_err(|e| {
        MicrosoftLoginError::new(format!("解析 MC Token 响应失败: {}", e)).with_step("mc_token")
    })?;

    log_info!("Minecraft token obtained successfully");
    Ok(result)
}

/// 步骤 5：验证游戏所有权
async fn check_entitlements(mc_token: &str) -> Result<bool, MicrosoftLoginError> {
    log_info!("Checking game entitlements");

    let client = http::get_client();

    let response = client
        .get(MC_ENTITLEMENTS_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await
        .map_err(|e| {
            MicrosoftLoginError::new(format!("验证所有权请求失败: {}", e)).with_step("entitlements")
        })?;

    if !response.status().is_success() {
        log_warn!("Entitlements check failed: {}", response.status());
        return Ok(false);
    }

    let body: serde_json::Value = response.json().await.map_err(|e| {
        MicrosoftLoginError::new(format!("解析所有权响应失败: {}", e)).with_step("entitlements")
    })?;

    let has_game = body
        .get("items")
        .and_then(|items| items.as_array())
        .map(|items| !items.is_empty())
        .unwrap_or(false);

    if !has_game {
        return Err(MicrosoftLoginError::new(
            "未检测到 Minecraft 游戏所有权，可能未购买正版或 Xbox Game Pass 已到期",
        )
        .with_step("entitlements"));
    }

    log_info!("Game ownership verified");
    Ok(true)
}

/// 步骤 6：获取玩家档案
async fn fetch_profile(mc_token: &str) -> Result<MinecraftProfile, MicrosoftLoginError> {
    log_info!("Fetching Minecraft profile");

    let client = http::get_client();

    let response = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await
        .map_err(|e| {
            MicrosoftLoginError::new(format!("获取档案请求失败: {}", e)).with_step("profile")
        })?;

    let status = response.status();

    if status.as_u16() == 429 {
        return Err(MicrosoftLoginError::new("请求太过频繁，请稍后再试")
            .with_step("profile")
            .with_code("429"));
    }

    if status.as_u16() == 404 {
        return Err(MicrosoftLoginError::new(
            "未找到 Minecraft 玩家档案，请访问 https://www.minecraft.net 创建档案",
        )
        .with_step("profile")
        .with_code("404"));
    }

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(
            MicrosoftLoginError::new(format!("获取档案失败 (HTTP {}): {}", status, body))
                .with_step("profile"),
        );
    }

    let profile: MinecraftProfile = response.json().await.map_err(|e| {
        MicrosoftLoginError::new(format!("解析档案响应失败: {}", e)).with_step("profile")
    })?;

    log_info!(
        "Profile obtained: username={}, uuid={}",
        profile.name,
        profile.id
    );
    Ok(profile)
}

/// 完成步骤 2-6（从 OAuth Token 开始的后续交换链）
///
/// 在获取到 OAuth Access Token 后，依次执行：
/// XBL → XSTS → MC Token → 验证所有权 → 获取档案
pub async fn complete_login_chain(
    oauth_access_token: &str,
    oauth_refresh_token: &str,
) -> Result<MicrosoftLoginResult, MicrosoftLoginError> {
    // 步骤 2: OAuth → XBL
    let xbl_response = exchange_xbl_token(oauth_access_token).await?;

    // 步骤 3: XBL → XSTS + UHS
    let (xsts_token, uhs) = exchange_xsts_token(&xbl_response.token).await?;

    // 步骤 4: XSTS → MC Token
    let mc_response = exchange_mc_token(&xsts_token, &uhs).await?;

    // 步骤 5: 验证游戏所有权
    check_entitlements(&mc_response.access_token).await?;

    // 步骤 6: 获取玩家档案
    let profile = fetch_profile(&mc_response.access_token).await?;

    // 计算过期时间（提前 20 分钟，与 PCL2 一致）
    let expires_at = unix_now() + mc_response.expires_in.saturating_sub(1200);

    let profile_json = serde_json::to_string(&profile).unwrap_or_default();

    Ok(MicrosoftLoginResult {
        username: profile.name,
        uuid: profile.id,
        access_token: mc_response.access_token,
        refresh_token: oauth_refresh_token.to_string(),
        expires_at,
        profile_json,
    })
}

/// 使用 Refresh Token 完成静默刷新（无需用户交互）
///
/// 流程：刷新 OAuth Token → XBL → XSTS → MC Token → 验证 → 档案
pub async fn login_with_refresh_token(
    refresh_token: &str,
) -> Result<MicrosoftLoginResult, MicrosoftLoginError> {
    log_info!("Attempting silent login with refresh token");

    let oauth_response = refresh_oauth_token(refresh_token).await?;

    // 使用新的 refresh_token（如果返回了的话），否则用原来的
    let new_refresh = oauth_response
        .refresh_token
        .as_deref()
        .unwrap_or(refresh_token);

    complete_login_chain(&oauth_response.access_token, new_refresh).await
}

/// 检查 Token 是否已过期
pub fn is_token_expired(expires_at: u64) -> bool {
    unix_now() >= expires_at
}

/// 获取轮询间隔（秒）
/// PCL2: 等待 interval - 1 秒后开始，每次失败间隔 2 秒
pub fn get_poll_interval(server_interval: u64) -> Duration {
    if server_interval > 1 {
        Duration::from_secs(server_interval - 1)
    } else {
        Duration::from_secs(2)
    }
}
