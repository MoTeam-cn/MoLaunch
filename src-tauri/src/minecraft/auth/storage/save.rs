//! `AuthStorage::save` 实现
//!
//! 将 `PersistedAuthState` 全部字段写入 Windows 注册表（敏感字段先加密）。
//! 先清除所有旧值，再写入新值，确保数据一致；写完后刷新内存缓存。

use crate::storage::registry::{reg_delete, reg_key, reg_set};

use super::registry::{
    ALL_KEYS, KEY_AUTHLIB_ACCOUNTS, KEY_AUTHLIB_CURRENT_ACCESS, KEY_AUTHLIB_CURRENT_CLIENT,
    KEY_AUTHLIB_CURRENT_NAME, KEY_AUTHLIB_CURRENT_SERVER_NAME, KEY_AUTHLIB_CURRENT_SERVER_URL,
    KEY_AUTHLIB_CURRENT_UUID, KEY_LEGACY_NAME, KEY_LEGACY_UUID, KEY_LOGIN_TYPE, KEY_MS_ACCOUNTS,
    KEY_MS_CURRENT_ACCESS, KEY_MS_CURRENT_EXPIRES, KEY_MS_CURRENT_NAME, KEY_MS_CURRENT_PROFILE,
    KEY_MS_CURRENT_REFRESH, KEY_MS_CURRENT_UUID, KEY_OFFLINE_ACCOUNTS,
};
use super::types::PersistedAuthState;
use super::AuthStorage;

impl AuthStorage {
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
                        self.reg_set_encrypted(
                            &key,
                            KEY_AUTHLIB_CURRENT_ACCESS,
                            &user.access_token,
                        )
                        .await?;
                        self.reg_set_encrypted(
                            &key,
                            KEY_AUTHLIB_CURRENT_CLIENT,
                            &user.client_token,
                        )
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

            // 写入多账号列表（通过 to_storage_json 手动序列化，避免派生 Serialize 误暴露 token）
            if !state.ms_accounts.is_empty() {
                let arr: Vec<serde_json::Value> = state
                    .ms_accounts
                    .iter()
                    .map(|a| a.to_storage_json())
                    .collect();
                let json = serde_json::to_string(&arr)
                    .map_err(|e| format!("序列化账号列表失败: {}", e))?;
                self.reg_set_encrypted(&key, KEY_MS_ACCOUNTS, &json).await?;
            }

            // 写入离线账号列表（无敏感字段，可直接序列化）
            if !state.offline_accounts.is_empty() {
                let json = serde_json::to_string(&state.offline_accounts)
                    .map_err(|e| format!("序列化离线账号列表失败: {}", e))?;
                self.reg_set_encrypted(&key, KEY_OFFLINE_ACCOUNTS, &json)
                    .await?;
            }

            // 写入 authlib 账号列表（通过 to_storage_json 手动序列化，避免派生 Serialize 误暴露 password/token）
            if !state.authlib_accounts.is_empty() {
                let arr: Vec<serde_json::Value> = state
                    .authlib_accounts
                    .iter()
                    .map(|a| a.to_storage_json())
                    .collect();
                let json = serde_json::to_string(&arr)
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
