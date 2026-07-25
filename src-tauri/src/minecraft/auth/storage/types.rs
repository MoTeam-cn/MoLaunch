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

/// 持久化的 yggdrasil（authlib-injector 外置登录）账号信息
///
/// 每个账号绑定一个 yggdrasil 服务器（server_url），与微软/离线账号平级。
/// 密码以加密形式存储（注册表整体加密），用于 token 失效后自动重新登录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAuthlibAccount {
    /// 登录账号（邮箱或用户名）
    pub username: String,
    /// 登录密码（明文，由 AuthStorage 整体加密后写入注册表）
    pub password: String,
    /// access_token（yggdrasil 令牌）
    pub access_token: String,
    /// client_token（客户端令牌）
    pub client_token: String,
    /// 选中的角色 UUID（profile.id）
    pub uuid: String,
    /// 选中的角色名（profile.name）
    pub player_name: String,
    /// yggdrasil API 根地址（如 `https://littleskin.cn/api/yggdrasil`）
    pub server_url: String,
    /// 服务器显示名（从元数据获取，如 `LittleSkin`）
    pub server_name: String,
}

/// 持久化的认证状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedAuthState {
    /// 当前登录的账号（离线/微软/authlib）
    pub current_user: Option<CurrentUser>,
    /// 已保存的微软账号列表（多账号）
    pub ms_accounts: Vec<StoredMsAccount>,
    /// 已保存的离线账号列表（多账号）
    #[serde(default)]
    pub offline_accounts: Vec<StoredOfflineAccount>,
    /// 已保存的 yggdrasil 外置登录账号列表（多账号）
    #[serde(default)]
    pub authlib_accounts: Vec<StoredAuthlibAccount>,
}

/// 当前登录用户（持久化用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub name: String,
    pub uuid: String,
    pub access_token: String,
    pub client_token: String,
    /// "Legacy" / "Microsoft" / "AuthlibInjector"
    pub login_type: String,
    pub profile_json: Option<String>,
    /// 微软登录的刷新令牌（仅微软登录有）
    pub refresh_token: Option<String>,
    /// 微软登录的过期时间戳（仅微软登录有）
    pub expires_at: Option<u64>,
    /// authlib 登录的 yggdrasil API 根地址（仅 authlib 登录有）
    /// 启动游戏时用于构建 -Dauthlibinjector.yggdrasil.prefetched 参数
    #[serde(default)]
    pub server_url: Option<String>,
    /// authlib 登录的服务器显示名（仅 authlib 登录有，用于 UI 展示）
    #[serde(default)]
    pub server_name: Option<String>,
}
