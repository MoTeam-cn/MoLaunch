//! 认证持久化数据结构

use serde::{Deserialize, Serialize};

use super::super::microsoft::MicrosoftLoginResult;

/// 持久化的微软账号信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMsAccount {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub profile_json: String,
}

impl From<&MicrosoftLoginResult> for StoredMsAccount {
    fn from(result: &MicrosoftLoginResult) -> Self {
        Self {
            username: result.username.clone(),
            uuid: result.uuid.clone(),
            access_token: result.access_token.clone(),
            refresh_token: result.refresh_token.clone(),
            expires_at: result.expires_at,
            profile_json: result.profile_json.clone(),
        }
    }
}

/// 持久化的离线账号信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredOfflineAccount {
    pub username: String,
    pub uuid: String,
    /// 用户选择的本地皮肤名称（None 表示使用默认 hash 皮肤）
    #[serde(default)]
    pub skin: Option<String>,
}

/// 持久化的认证状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedAuthState {
    /// 当前登录的账号（离线或微软）
    pub current_user: Option<CurrentUser>,
    /// 已保存的微软账号列表（多账号）
    pub ms_accounts: Vec<StoredMsAccount>,
    /// 已保存的离线账号列表（多账号）
    #[serde(default)]
    pub offline_accounts: Vec<StoredOfflineAccount>,
}

/// 当前登录用户（持久化用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub name: String,
    pub uuid: String,
    pub access_token: String,
    pub client_token: String,
    /// "Legacy" 或 "Microsoft"
    pub login_type: String,
    pub profile_json: Option<String>,
    /// 微软登录的刷新令牌（仅微软登录有）
    pub refresh_token: Option<String>,
    /// 微软登录的过期时间戳（仅微软登录有）
    pub expires_at: Option<u64>,
}
