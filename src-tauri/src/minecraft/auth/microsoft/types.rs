//! 微软登录数据结构
//!
//! 安全约束（方案 C）：所有结构体仅派生 `Deserialize`（解析 HTTP 响应用），
//! **不派生 `Serialize`**，避免 `serde_json::to_value` 误将 token 字段暴露到 IPC。
//! 这些结构体仅在 `microsoft/` 内部模块间传递，持久化由 `StoredMsAccount` 接管。

use serde::Deserialize;

/// 设备码响应（v2.0 Device Code Flow）
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

/// OAuth Token 响应
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: Option<String>,
}

/// XBL Token 响应
///
/// Microsoft XBL/XSTS 端点返回 PascalCase 字段名，必须用 `#[serde(rename)]` 映射。
#[derive(Debug, Clone, Deserialize)]
pub struct XblTokenResponse {
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "IssueInstant", default)]
    pub issue_instant: Option<String>,
    #[serde(rename = "NotAfter", default)]
    pub not_after: Option<String>,
    #[serde(rename = "DisplayClaims", default)]
    pub display_claims: Option<serde_json::Value>,
}

/// XSTS Token 响应
#[derive(Debug, Clone, Deserialize)]
pub struct XstsTokenResponse {
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "IssueInstant", default)]
    pub issue_instant: Option<String>,
    #[serde(rename = "NotAfter", default)]
    pub not_after: Option<String>,
    #[serde(rename = "DisplayClaims", default)]
    pub display_claims: Option<serde_json::Value>,
}

/// Minecraft 登录响应
#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftLoginResponse {
    pub username: String,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Minecraft 玩家档案
///
/// 仅含 id/name/skins/capes（皮肤披风 URL），不含 token。
/// `exchange.rs::login_with_xbl` 用 `to_string(&profile)` 构建 `profile_json` 字符串存入
/// `MicrosoftLoginResult.profile_json`，最终经 `LocalAuthResult.profile_json` 返回前端用于头像显示。
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub skins: Vec<serde_json::Value>,
    #[serde(default)]
    pub capes: Vec<serde_json::Value>,
}

/// 完整的微软登录结果（用于持久化）
///
/// 含 `access_token` / `refresh_token`，仅派生 `Deserialize`。
/// 持久化时由 `StoredMsAccount::from(&MicrosoftLoginResult)` 转换后通过 `to_storage_json()` 写入。
#[derive(Debug, Clone, Deserialize)]
pub struct MicrosoftLoginResult {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub profile_json: String,
    /// Xbox 用户 ID（XSTS DisplayClaims.xui[0].xui），用于启动参数 `--xuid`
    pub xuid: String,
}

/// 微软登录错误
#[derive(Debug, Clone, Deserialize)]
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
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_code: None,
            step: None,
        }
    }

    pub fn with_step(mut self, step: impl Into<String>) -> Self {
        self.step = Some(step.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }
}
