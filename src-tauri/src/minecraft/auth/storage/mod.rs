//! 认证持久化模块
//!
//! 将认证信息存储到 Windows 注册表：
//! - 每个字段单独存储为注册表键值（而非单个 JSON 文件）
//! - 敏感字段（Token、用户名、UUID 等）使用 SDK DES 加密
//! - 非敏感字段（登录类型）明文存储
//! - 多账号列表用一个加密的 JSON 字符串存储
//!
//! 注册表路径：`HKEY_CURRENT_USER\Software\MoLaunch`
//!
//! 按关注点拆分为 4 个子模块：
//! - `types`      数据结构（StoredMsAccount / StoredOfflineAccount / PersistedAuthState / CurrentUser）
//! - `registry`   注册表常量 + 低层 reg_key/reg_get/reg_set/reg_delete 自由函数
//! - `operations` AuthStorage 高层操作（save_ms_login / save_offline_login / token 刷新等 11 个方法）
//! - `mod.rs`     AuthStorage 结构体 + encrypt/decrypt + reg_set_encrypted/reg_get_decrypted + load/invalidate/save

mod operations;
mod registry;
mod types;

pub use types::{
    CurrentUser, PersistedAuthState, StoredAuthlibAccount, StoredMsAccount, StoredOfflineAccount,
};

use crate::log_info;
use crate::sdk::SdkInstance;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use crate::storage::registry::{reg_delete, reg_get, reg_key, reg_set};
use registry::{
    ALL_KEYS, KEY_AUTHLIB_ACCOUNTS, KEY_AUTHLIB_CURRENT_ACCESS, KEY_AUTHLIB_CURRENT_CLIENT,
    KEY_AUTHLIB_CURRENT_NAME, KEY_AUTHLIB_CURRENT_SERVER_NAME, KEY_AUTHLIB_CURRENT_SERVER_URL,
    KEY_AUTHLIB_CURRENT_UUID, KEY_LEGACY_NAME, KEY_LEGACY_UUID, KEY_LOGIN_TYPE, KEY_MS_ACCOUNTS,
    KEY_MS_CURRENT_ACCESS, KEY_MS_CURRENT_EXPIRES, KEY_MS_CURRENT_NAME, KEY_MS_CURRENT_PROFILE,
    KEY_MS_CURRENT_REFRESH, KEY_MS_CURRENT_UUID, KEY_OFFLINE_ACCOUNTS,
};

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
    async fn reg_set_encrypted(
        &self,
        key: &winreg::RegKey,
        name: &str,
        value: &str,
    ) -> Result<(), String> {
        let encrypted = self.encrypt(value).await?;
        reg_set(key, name, &encrypted)
    }

    /// 读取并解密注册表
    #[cfg(windows)]
    async fn reg_get_decrypted(&self, key: &winreg::RegKey, name: &str) -> Option<String> {
        let encrypted = reg_get(key, name)?;
        self.decrypt(&encrypted).await.ok()
    }

    // --------------------------------------------------------
    // 公开 API：load / invalidate / save
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
            let key = reg_key()?;
            let mut state = PersistedAuthState::default();

            // 读取登录类型
            let login_type = reg_get(&key, KEY_LOGIN_TYPE).unwrap_or_default();

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
                        server_url: None,
                        server_name: None,
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
                        server_url: None,
                        server_name: None,
                    });
                }
            } else if login_type == "AuthlibInjector" {
                // authlib-injector 外置登录
                let name = self
                    .reg_get_decrypted(&key, KEY_AUTHLIB_CURRENT_NAME)
                    .await
                    .unwrap_or_default();
                let uuid = self
                    .reg_get_decrypted(&key, KEY_AUTHLIB_CURRENT_UUID)
                    .await
                    .unwrap_or_default();
                if !name.is_empty() {
                    let access = self
                        .reg_get_decrypted(&key, KEY_AUTHLIB_CURRENT_ACCESS)
                        .await
                        .unwrap_or_default();
                    let client = self
                        .reg_get_decrypted(&key, KEY_AUTHLIB_CURRENT_CLIENT)
                        .await
                        .unwrap_or_default();
                    let server_url = self
                        .reg_get_decrypted(&key, KEY_AUTHLIB_CURRENT_SERVER_URL)
                        .await;
                    let server_name = self
                        .reg_get_decrypted(&key, KEY_AUTHLIB_CURRENT_SERVER_NAME)
                        .await;

                    state.current_user = Some(CurrentUser {
                        name,
                        uuid,
                        access_token: access,
                        client_token: client,
                        login_type: "AuthlibInjector".to_string(),
                        profile_json: None,
                        refresh_token: None,
                        expires_at: None,
                        server_url,
                        server_name,
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
                    state.offline_accounts =
                        serde_json::from_str(&offline_json).unwrap_or_default();
                }
            }

            // 读取 authlib 账号列表
            if let Some(authlib_json) = self.reg_get_decrypted(&key, KEY_AUTHLIB_ACCOUNTS).await {
                if !authlib_json.is_empty() {
                    state.authlib_accounts =
                        serde_json::from_str(&authlib_json).unwrap_or_default();
                }
            }

            log_info!(
                "Loaded persisted auth state: current_user={}, ms_accounts={}, offline_accounts={}, authlib_accounts={}",
                state
                    .current_user
                    .as_ref()
                    .map(|u| u.name.as_str())
                    .unwrap_or("none"),
                state.ms_accounts.len(),
                state.offline_accounts.len(),
                state.authlib_accounts.len()
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
            let key = reg_key()?;

            // 清除所有旧值
            for name in ALL_KEYS {
                let _ = reg_delete(&key, name);
            }

            // 写入当前用户
            if let Some(ref user) = state.current_user {
                // 登录类型（明文）
                reg_set(&key, KEY_LOGIN_TYPE, &user.login_type)?;

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
                    "AuthlibInjector" => {
                        self.reg_set_encrypted(&key, KEY_AUTHLIB_CURRENT_NAME, &user.name)
                            .await?;
                        self.reg_set_encrypted(&key, KEY_AUTHLIB_CURRENT_UUID, &user.uuid)
                            .await?;
                        self.reg_set_encrypted(&key, KEY_AUTHLIB_CURRENT_ACCESS, &user.access_token)
                            .await?;
                        self.reg_set_encrypted(&key, KEY_AUTHLIB_CURRENT_CLIENT, &user.client_token)
                            .await?;
                        if let Some(ref server_url) = user.server_url {
                            self.reg_set_encrypted(
                                &key,
                                KEY_AUTHLIB_CURRENT_SERVER_URL,
                                server_url,
                            )
                            .await?;
                        }
                        if let Some(ref server_name) = user.server_name {
                            self.reg_set_encrypted(
                                &key,
                                KEY_AUTHLIB_CURRENT_SERVER_NAME,
                                server_name,
                            )
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
                self.reg_set_encrypted(&key, KEY_OFFLINE_ACCOUNTS, &json)
                    .await?;
            }

            // 写入 authlib 账号列表
            if !state.authlib_accounts.is_empty() {
                let json = serde_json::to_string(&state.authlib_accounts)
                    .map_err(|e| format!("序列化 authlib 账号列表失败: {}", e))?;
                self.reg_set_encrypted(&key, KEY_AUTHLIB_ACCOUNTS, &json)
                    .await?;
            }

            // 更新内存缓存
            *self.cache.lock().await = Some(state.clone());

            Ok(())
        }
    }
}
