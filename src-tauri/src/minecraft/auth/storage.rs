//! 认证持久化模块
//!
//! 参考 PCL2 的存储方式，将认证信息存储到 Windows 注册表：
//! - 每个字段单独存储为注册表键值（而非单个 JSON 文件）
//! - 敏感字段（Token、用户名、UUID 等）使用 SDK DES 加密
//! - 非敏感字段（登录类型）明文存储
//! - 多账号列表用一个加密的 JSON 字符串存储
//!
//! 注册表路径：`HKEY_CURRENT_USER\Software\MoLaunch`

use crate::log_info;
use crate::sdk::SdkInstance;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

use super::microsoft::MicrosoftLoginResult;

// ============================================================
// 注册表键名定义（参考 PCL2 的命名风格）
// ============================================================

/// 注册表子键路径
const REG_SUBKEY: &str = "Software\\MoLaunch";

/// 登录类型（明文）："Legacy" 或 "Microsoft"
const KEY_LOGIN_TYPE: &str = "LoginType";

/// 离线登录用户名（加密）
const KEY_LEGACY_NAME: &str = "LoginLegacyName";
/// 离线登录 UUID（加密）
const KEY_LEGACY_UUID: &str = "LoginLegacyUuid";

/// 当前微软账号用户名（加密）
const KEY_MS_CURRENT_NAME: &str = "MsCurrentName";
/// 当前微软账号 UUID（加密）
const KEY_MS_CURRENT_UUID: &str = "MsCurrentUuid";
/// 当前微软账号 access_token（加密）
const KEY_MS_CURRENT_ACCESS: &str = "MsCurrentAccess";
/// 当前微软账号 refresh_token（加密）
const KEY_MS_CURRENT_REFRESH: &str = "MsCurrentRefresh";
/// 当前微软账号过期时间戳（加密，字符串形式的 u64）
const KEY_MS_CURRENT_EXPIRES: &str = "MsCurrentExpires";
/// 当前微软账号档案 JSON（加密）
const KEY_MS_CURRENT_PROFILE: &str = "MsCurrentProfile";

/// 所有微软账号列表 JSON（加密）
const KEY_MS_ACCOUNTS: &str = "MsAccounts";

/// 所有离线账号列表 JSON（加密）
const KEY_OFFLINE_ACCOUNTS: &str = "OfflineAccounts";

/// 所有注册表键名（用于清理）
#[cfg(windows)]
const ALL_KEYS: &[&str] = &[
    KEY_LOGIN_TYPE,
    KEY_LEGACY_NAME,
    KEY_LEGACY_UUID,
    KEY_MS_CURRENT_NAME,
    KEY_MS_CURRENT_UUID,
    KEY_MS_CURRENT_ACCESS,
    KEY_MS_CURRENT_REFRESH,
    KEY_MS_CURRENT_EXPIRES,
    KEY_MS_CURRENT_PROFILE,
    KEY_MS_ACCOUNTS,
    KEY_OFFLINE_ACCOUNTS,
];

// ============================================================
// 数据结构（保持向后兼容，对外 API 不变）
// ============================================================

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

// ============================================================
// 认证存储管理器
// ============================================================

/// 认证存储管理器
///
/// 使用 Windows 注册表存储认证信息，每个字段单独存储。
/// 敏感字段使用 SDK DES 加密，非敏感字段明文存储。
pub struct AuthStorage {
    /// SDK 实例引用（用于 DES 加解密）
    sdk: Arc<TokioMutex<Option<SdkInstance>>>,
    /// 内存缓存（避免每次命令都重新读注册表+解密+打日志）
    /// save 系列方法会自动刷新此缓存
    cache: TokioMutex<Option<PersistedAuthState>>,
}

impl AuthStorage {
    pub fn new(sdk: Arc<TokioMutex<Option<SdkInstance>>>) -> Self {
        Self {
            sdk,
            cache: TokioMutex::new(None),
        }
    }

    // --------------------------------------------------------
    // 注册表操作工具
    // --------------------------------------------------------

    /// 打开或创建注册表子键
    #[cfg(windows)]
    fn reg_key() -> Result<RegKey, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey_with_flags(REG_SUBKEY, KEY_SET_VALUE | KEY_READ)
            .or_else(|_| {
                hkcu.create_subkey(REG_SUBKEY)
                    .map(|(k, _)| k)
                    .map_err(|e| e.to_string())
            })
            .map_err(|e| format!("打开注册表失败: {}", e))
    }

    /// 读取注册表字符串值
    #[cfg(windows)]
    fn reg_get(key: &RegKey, name: &str) -> Option<String> {
        key.get_value::<String, _>(name).ok()
    }

    /// 写入注册表字符串值
    #[cfg(windows)]
    fn reg_set(key: &RegKey, name: &str, value: &str) -> Result<(), String> {
        key.set_value(name, &value)
            .map_err(|e| format!("写入注册表失败: {}", e))
    }

    /// 删除注册表值（不存在不算错误）
    #[cfg(windows)]
    fn reg_delete(key: &RegKey, name: &str) -> Result<(), String> {
        match key.delete_value(name) {
            Ok(()) => Ok(()),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("删除注册表失败: {}", e)),
        }
    }

    // --------------------------------------------------------
    // 加解密工具
    // --------------------------------------------------------

    /// 加密数据（使用 SDK 内置的 DES 加密）
    async fn encrypt(&self, data: &str) -> Result<String, String> {
        let sdk = self.sdk.lock().await;
        match sdk.as_ref() {
            Some(sdk) => sdk
                .encrypt_token(data)
                .map_err(|e| format!("加密失败: {}", e)),
            None => Err("SDK 未加载，无法加密认证数据".to_string()),
        }
    }

    /// 解密数据（使用 SDK 内置的 DES 解密）
    async fn decrypt(&self, data: &str) -> Result<String, String> {
        let sdk = self.sdk.lock().await;
        match sdk.as_ref() {
            Some(sdk) => sdk
                .decrypt_token(data)
                .map_err(|e| format!("解密失败: {}", e)),
            None => Err("SDK 未加载，无法解密认证数据".to_string()),
        }
    }

    /// 加密并写入注册表
    #[cfg(windows)]
    async fn reg_set_encrypted(&self, key: &RegKey, name: &str, value: &str) -> Result<(), String> {
        let encrypted = self.encrypt(value).await?;
        Self::reg_set(key, name, &encrypted)
    }

    /// 读取并解密注册表
    #[cfg(windows)]
    async fn reg_get_decrypted(&self, key: &RegKey, name: &str) -> Option<String> {
        let encrypted = Self::reg_get(key, name)?;
        self.decrypt(&encrypted).await.ok()
    }

    // --------------------------------------------------------
    // 公开 API
    // --------------------------------------------------------

    /// 加载持久化的认证状态
    ///
    /// 优先返回内存缓存，避免每次命令都重新读注册表+解密+打日志。
    /// save 系列方法会自动刷新缓存；如需强制从注册表读取，调用 `invalidate` 后再 load。
    pub async fn load(&self) -> Result<PersistedAuthState, String> {
        // 优先返回缓存
        if let Some(cached) = self.cache.lock().await.clone() {
            return Ok(cached);
        }

        #[cfg(not(windows))]
        {
            let state = PersistedAuthState::default();
            *self.cache.lock().await = Some(state.clone());
            return Ok(state);
        }

        #[cfg(windows)]
        {
            let key = Self::reg_key()?;
            let mut state = PersistedAuthState::default();

            // 读取登录类型
            let login_type = Self::reg_get(&key, KEY_LOGIN_TYPE).unwrap_or_default();

            if login_type == "Legacy" {
                // 离线登录
                let name = self
                    .reg_get_decrypted(&key, KEY_LEGACY_NAME)
                    .await
                    .unwrap_or_default();
                let uuid = self
                    .reg_get_decrypted(&key, KEY_LEGACY_UUID)
                    .await
                    .unwrap_or_default();
                if !name.is_empty() {
                    state.current_user = Some(CurrentUser {
                        name,
                        uuid: uuid.clone(),
                        access_token: uuid.clone(),
                        client_token: uuid,
                        login_type: "Legacy".to_string(),
                        profile_json: None,
                        refresh_token: None,
                        expires_at: None,
                    });
                }
            } else if login_type == "Microsoft" {
                // 微软登录
                let name = self
                    .reg_get_decrypted(&key, KEY_MS_CURRENT_NAME)
                    .await
                    .unwrap_or_default();
                let uuid = self
                    .reg_get_decrypted(&key, KEY_MS_CURRENT_UUID)
                    .await
                    .unwrap_or_default();
                if !name.is_empty() {
                    let access = self
                        .reg_get_decrypted(&key, KEY_MS_CURRENT_ACCESS)
                        .await
                        .unwrap_or_default();
                    let refresh = self
                        .reg_get_decrypted(&key, KEY_MS_CURRENT_REFRESH)
                        .await
                        .unwrap_or_default();
                    let expires = self
                        .reg_get_decrypted(&key, KEY_MS_CURRENT_EXPIRES)
                        .await
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);
                    let profile = self.reg_get_decrypted(&key, KEY_MS_CURRENT_PROFILE).await;

                    state.current_user = Some(CurrentUser {
                        name,
                        uuid,
                        access_token: access,
                        client_token: String::new(),
                        login_type: "Microsoft".to_string(),
                        profile_json: profile,
                        refresh_token: Some(refresh),
                        expires_at: Some(expires),
                    });
                }
            }

            // 读取多账号列表
            if let Some(accounts_json) = self.reg_get_decrypted(&key, KEY_MS_ACCOUNTS).await {
                if !accounts_json.is_empty() {
                    state.ms_accounts = serde_json::from_str(&accounts_json).unwrap_or_default();
                }
            }

            // 读取离线账号列表
            if let Some(offline_json) = self.reg_get_decrypted(&key, KEY_OFFLINE_ACCOUNTS).await {
                if !offline_json.is_empty() {
                    state.offline_accounts = serde_json::from_str(&offline_json).unwrap_or_default();
                }
            }

            log_info!(
                "Loaded persisted auth state: current_user={}, ms_accounts={}, offline_accounts={}",
                state
                    .current_user
                    .as_ref()
                    .map(|u| u.name.as_str())
                    .unwrap_or("none"),
                state.ms_accounts.len(),
                state.offline_accounts.len()
            );

            // 写入缓存，后续 load 直接返回
            *self.cache.lock().await = Some(state.clone());

            Ok(state)
        }
    }

    /// 清除内存缓存，强制下次 load 从注册表重新读取
    pub async fn invalidate(&self) {
        *self.cache.lock().await = None;
    }

    /// 保存认证状态到注册表
    ///
    /// 将 `PersistedAuthState` 的所有字段写入注册表。
    /// 先清除所有旧值，再写入新值，确保数据一致。
    pub async fn save(&self, state: &PersistedAuthState) -> Result<(), String> {
        #[cfg(not(windows))]
        {
            let _ = state;
            return Ok(());
        }

        #[cfg(windows)]
        {
            let key = Self::reg_key()?;

            // 清除所有旧值
            for name in ALL_KEYS {
                let _ = Self::reg_delete(&key, name);
            }

            // 写入当前用户
            if let Some(ref user) = state.current_user {
                // 登录类型（明文）
                Self::reg_set(&key, KEY_LOGIN_TYPE, &user.login_type)?;

                match user.login_type.as_str() {
                    "Legacy" => {
                        self.reg_set_encrypted(&key, KEY_LEGACY_NAME, &user.name)
                            .await?;
                        self.reg_set_encrypted(&key, KEY_LEGACY_UUID, &user.uuid)
                            .await?;
                    }
                    "Microsoft" => {
                        self.reg_set_encrypted(&key, KEY_MS_CURRENT_NAME, &user.name)
                            .await?;
                        self.reg_set_encrypted(&key, KEY_MS_CURRENT_UUID, &user.uuid)
                            .await?;
                        self.reg_set_encrypted(&key, KEY_MS_CURRENT_ACCESS, &user.access_token)
                            .await?;
                        if let Some(ref refresh) = user.refresh_token {
                            self.reg_set_encrypted(&key, KEY_MS_CURRENT_REFRESH, refresh)
                                .await?;
                        }
                        if let Some(expires) = user.expires_at {
                            self.reg_set_encrypted(
                                &key,
                                KEY_MS_CURRENT_EXPIRES,
                                &expires.to_string(),
                            )
                            .await?;
                        }
                        if let Some(ref profile) = user.profile_json {
                            self.reg_set_encrypted(&key, KEY_MS_CURRENT_PROFILE, profile)
                                .await?;
                        }
                    }
                    _ => {}
                }
            }

            // 写入多账号列表
            if !state.ms_accounts.is_empty() {
                let json = serde_json::to_string(&state.ms_accounts)
                    .map_err(|e| format!("序列化账号列表失败: {}", e))?;
                self.reg_set_encrypted(&key, KEY_MS_ACCOUNTS, &json).await?;
            }

            // 写入离线账号列表
            if !state.offline_accounts.is_empty() {
                let json = serde_json::to_string(&state.offline_accounts)
                    .map_err(|e| format!("序列化离线账号列表失败: {}", e))?;
                self.reg_set_encrypted(&key, KEY_OFFLINE_ACCOUNTS, &json).await?;
            }

            // 更新内存缓存
            *self.cache.lock().await = Some(state.clone());

            Ok(())
        }
    }

    /// 保存微软登录结果并设为当前用户
    pub async fn save_ms_login(&self, result: &MicrosoftLoginResult) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 更新或添加到微软账号列表
        let account = StoredMsAccount::from(result);
        if let Some(existing) = state
            .ms_accounts
            .iter_mut()
            .find(|a| a.uuid == account.uuid)
        {
            *existing = account.clone();
        } else {
            state.ms_accounts.push(account.clone());
        }

        // 设为当前用户
        state.current_user = Some(CurrentUser {
            name: result.username.clone(),
            uuid: result.uuid.clone(),
            access_token: result.access_token.clone(),
            client_token: String::new(),
            login_type: "Microsoft".to_string(),
            profile_json: Some(result.profile_json.clone()),
            refresh_token: Some(result.refresh_token.clone()),
            expires_at: Some(result.expires_at),
        });

        self.save(&state).await
    }

    /// 保存离线登录并设为当前用户
    ///
    /// 同时把账号添加到离线账号列表（UUID 去重），保留已有的 skin 选择。
    pub async fn save_offline_login(&self, username: &str, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 添加到离线账号列表（UUID 去重，保留已有的 skin 选择）
        let account = StoredOfflineAccount {
            username: username.to_string(),
            uuid: uuid.to_string(),
            skin: state
                .offline_accounts
                .iter()
                .find(|a| a.uuid == uuid)
                .and_then(|a| a.skin.clone()),
        };
        if let Some(existing) = state
            .offline_accounts
            .iter_mut()
            .find(|a| a.uuid == account.uuid)
        {
            *existing = account.clone();
        } else {
            state.offline_accounts.push(account);
        }

        state.current_user = Some(CurrentUser {
            name: username.to_string(),
            uuid: uuid.to_string(),
            access_token: uuid.to_string(),
            client_token: uuid.to_string(),
            login_type: "Legacy".to_string(),
            profile_json: None,
            refresh_token: None,
            expires_at: None,
        });

        self.save(&state).await
    }

    /// 设置离线账号的皮肤选择
    pub async fn set_offline_skin(&self, uuid: &str, skin: Option<&str>) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        if let Some(account) = state.offline_accounts.iter_mut().find(|a| a.uuid == uuid) {
            account.skin = skin.map(|s| s.to_string());
            self.save(&state).await
        } else {
            Err("离线账号不存在".to_string())
        }
    }

    /// 删除指定离线账号
    pub async fn remove_offline_account(&self, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.offline_accounts.retain(|a| a.uuid != uuid);

        // 如果删除的是当前用户，也清除当前用户
        if let Some(ref current) = state.current_user {
            if current.uuid == uuid {
                state.current_user = None;
            }
        }

        self.save(&state).await
    }

    /// 获取指定离线账号
    pub async fn get_offline_account(&self, uuid: &str) -> Result<Option<StoredOfflineAccount>, String> {
        let state = self.load().await?;
        Ok(state
            .offline_accounts
            .into_iter()
            .find(|a| a.uuid == uuid))
    }

    /// 清除当前用户（登出）
    pub async fn clear_current_user(&self) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.current_user = None;
        self.save(&state).await
    }

    /// 删除指定微软账号
    pub async fn remove_ms_account(&self, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.ms_accounts.retain(|a| a.uuid != uuid);

        // 如果删除的是当前用户，也清除当前用户
        if let Some(ref current) = state.current_user {
            if current.uuid == uuid {
                state.current_user = None;
            }
        }

        self.save(&state).await
    }

    /// 获取指定微软账号（用于刷新）
    pub async fn get_ms_account(&self, uuid: &str) -> Result<Option<StoredMsAccount>, String> {
        let state = self.load().await?;
        Ok(state.ms_accounts.into_iter().find(|a| a.uuid == uuid))
    }

    /// 获取当前微软账号的 refresh_token（用于静默刷新）
    pub async fn get_current_refresh_token(&self) -> Result<Option<String>, String> {
        let state = self.load().await?;
        match state.current_user {
            Some(ref user) if user.login_type == "Microsoft" => Ok(user.refresh_token.clone()),
            _ => Ok(None),
        }
    }

    /// 更新微软账号的 Token（刷新后调用）
    pub async fn update_ms_token(
        &self,
        uuid: &str,
        access_token: &str,
        refresh_token: &str,
        expires_at: u64,
    ) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 更新账号列表
        for account in state.ms_accounts.iter_mut() {
            if account.uuid == uuid {
                account.access_token = access_token.to_string();
                account.refresh_token = refresh_token.to_string();
                account.expires_at = expires_at;
                break;
            }
        }

        // 更新当前用户
        if let Some(ref mut current) = state.current_user {
            if current.uuid == uuid {
                current.access_token = access_token.to_string();
                current.refresh_token = Some(refresh_token.to_string());
                current.expires_at = Some(expires_at);
            }
        }

        self.save(&state).await
    }
}
