//! 认证持久化数据结构
//! 安全约束（方案 C：移除 `Serialize` derive 强制编译期阻止 IPC 误用）：
//! `StoredMsAccount`/`StoredAuthlibAccount`/`CurrentUser`/`PersistedAuthState` 仅派生
//! `Deserialize`（注册表加密 JSON 反序列化），不派生 `Serialize`；持久化通过 `to_storage_json()`
//! 手动构建 JSON 避免 `serde_json::to_value` 误将敏感字段序列化到 IPC。IPC 返回前端须用专用 View 结构体（敏感字段标 `#[serde(skip)]`）。

use serde::Deserialize;

use super::super::microsoft::MicrosoftLoginResult;

/// 持久化的微软账号信息
///
/// 含 `access_token` / `refresh_token` 敏感字段，仅派生 `Deserialize`（从注册表反序列化）。
/// 持久化写入时调用 `to_storage_json()`；IPC 返回前端用 `MsAccountInfo` 过滤。
#[derive(Debug, Clone, Deserialize)]
pub struct StoredMsAccount {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub profile_json: String,
}

impl StoredMsAccount {
    /// 构建包含全部字段（含 token）的 JSON，仅供持久化写入注册表使用
    pub fn to_storage_json(&self) -> serde_json::Value {
        serde_json::json!({
            "username": self.username,
            "uuid": self.uuid,
            "access_token": self.access_token,
            "refresh_token": self.refresh_token,
            "expires_at": self.expires_at,
            "profile_json": self.profile_json,
        })
    }
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
///
/// 无敏感字段（仅 username/uuid/skin），保留 `Serialize` 便于持久化。
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct StoredOfflineAccount {
    pub username: String,
    pub uuid: String,
    /// 用户选择的本地皮肤名称（None 表示使用默认 hash 皮肤）
    #[serde(default)]
    pub skin: Option<String>,
}

/// 持久化的 yggdrasil（authlib-injector 外置登录）账号信息
///
/// 含 `password` / `access_token` / `client_token` 敏感字段，仅派生 `Deserialize`。
/// 持久化写入时调用 `to_storage_json()`；IPC 返回前端用 `AuthlibAccountInfo` 过滤。
#[derive(Debug, Clone, Deserialize)]
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

impl StoredAuthlibAccount {
    /// 构建包含全部字段（含 password/token）的 JSON，仅供持久化写入注册表使用
    pub fn to_storage_json(&self) -> serde_json::Value {
        serde_json::json!({
            "username": self.username,
            "password": self.password,
            "access_token": self.access_token,
            "client_token": self.client_token,
            "uuid": self.uuid,
            "player_name": self.player_name,
            "server_url": self.server_url,
            "server_name": self.server_name,
        })
    }
}

/// 持久化的认证状态
///
/// 仅派生 `Deserialize` + `Default`（内存态由 `AuthStorage::load` 逐字段构造）。
/// `AuthStorage::save` 逐字段写入注册表，不依赖整体 `Serialize`。
#[derive(Debug, Clone, Deserialize, Default)]
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
///
/// 含 `access_token` / `client_token` / `refresh_token` 敏感字段，仅派生 `Deserialize`。
/// `AuthStorage::save` 通过 `to_storage_json()` 整体序列化到加密文件；IPC 返回前端用 `LocalAuthResult`（已 `#[serde(skip)]`）。
#[derive(Debug, Clone, Deserialize)]
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

impl CurrentUser {
    /// 构建包含全部字段（含 token）的 JSON，仅供持久化写入加密文件使用
    pub fn to_storage_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "uuid": self.uuid,
            "access_token": self.access_token,
            "client_token": self.client_token,
            "login_type": self.login_type,
            "profile_json": self.profile_json,
            "refresh_token": self.refresh_token,
            "expires_at": self.expires_at,
            "server_url": self.server_url,
            "server_name": self.server_name,
        })
    }
}

impl PersistedAuthState {
    /// 构建包含全部字段（含各账号列表的敏感字段）的 JSON，仅供持久化写入加密文件使用
    ///
    /// 通过手动构建 JSON 而非派生 `Serialize`，强制编译期阻止 IPC 误用敏感字段。
    pub fn to_storage_json(&self) -> serde_json::Value {
        serde_json::json!({
            "current_user": self.current_user.as_ref().map(|u| u.to_storage_json()),
            "ms_accounts": self.ms_accounts.iter().map(|a| a.to_storage_json()).collect::<Vec<_>>(),
            "offline_accounts": self.offline_accounts,
            "authlib_accounts": self.authlib_accounts.iter().map(|a| a.to_storage_json()).collect::<Vec<_>>(),
        })
    }
}
