//! Windows：注册表逐字段 SDK 解密读取

use crate::log_info;

use super::super::types::{CurrentUser, PersistedAuthState};
use super::super::AuthStorage;

impl AuthStorage {
    /// 从注册表加载认证状态（Windows）
    ///
    /// 读取注册表 `HKCU\Software\MoLaunch` 逐字段（敏感字段 SDK 解密），
    /// 构造 `PersistedAuthState`。注册表键不存在时返回 `PersistedAuthState::default()`。
    pub(super) async fn load_from_registry(&self) -> Result<PersistedAuthState, String> {
        use crate::storage::registry::{reg_get, reg_key};

        use super::super::registry::{
            KEY_AUTHLIB_ACCOUNTS, KEY_AUTHLIB_CURRENT_ACCESS, KEY_AUTHLIB_CURRENT_CLIENT,
            KEY_AUTHLIB_CURRENT_NAME, KEY_AUTHLIB_CURRENT_SERVER_NAME,
            KEY_AUTHLIB_CURRENT_SERVER_URL, KEY_AUTHLIB_CURRENT_UUID, KEY_LEGACY_NAME,
            KEY_LEGACY_UUID, KEY_LOGIN_TYPE, KEY_MS_ACCOUNTS, KEY_MS_CURRENT_ACCESS,
            KEY_MS_CURRENT_EXPIRES, KEY_MS_CURRENT_NAME, KEY_MS_CURRENT_PROFILE,
            KEY_MS_CURRENT_REFRESH, KEY_MS_CURRENT_UUID, KEY_MS_CURRENT_XUID, KEY_OFFLINE_ACCOUNTS,
        };

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
                    xuid: None,
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
                let xuid = self.reg_get_decrypted(&key, KEY_MS_CURRENT_XUID).await;

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
                    xuid: xuid.filter(|s| !s.is_empty()),
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
                    xuid: None,
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

        // 读取 authlib 账号列表
        if let Some(authlib_json) = self.reg_get_decrypted(&key, KEY_AUTHLIB_ACCOUNTS).await {
            if !authlib_json.is_empty() {
                state.authlib_accounts = serde_json::from_str(&authlib_json).unwrap_or_default();
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
