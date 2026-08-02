//! `AuthStorage::load` 实现：注册表 / JSON 文件双轨制读取（优先返回内存缓存）

use crate::log_info;

use super::types::{CurrentUser, PersistedAuthState};
use super::AuthStorage;

impl AuthStorage {
    /// 加载持久化的认证状态
    ///
    /// 优先返回内存缓存，避免每次命令都重新读存储+解密+打日志。
    /// save 系列方法会自动刷新缓存；如需强制从存储读取，调用 `invalidate` 后再 load。
    pub async fn load(&self) -> Result<PersistedAuthState, String> {
        // 优先返回缓存
        if let Some(cached) = self.cache.lock().await.clone() {
            return Ok(cached);
        }

        #[cfg(not(windows))]
        {
            self.load_from_file().await
        }

        #[cfg(windows)]
        {
            self.load_from_registry().await
        }
    }

    // ============================================================
    // 非 Windows：JSON 文件结构化逐字段解密读取
    // ============================================================

    /// 从 JSON 文件加载认证状态（非 Windows）
    ///
    /// 读取文件 → 解析为 `serde_json::Value` → 逐字段 SDK 解密敏感字段 →
    /// 构造 `PersistedAuthState`。文件不存在时返回 `PersistedAuthState::default()`，
    /// 解析失败时返回 Err。
    #[cfg(not(windows))]
    async fn load_from_file(&self) -> Result<PersistedAuthState, String> {
        use serde_json::Value;

        let path = crate::storage::appdata::appdata_root()?.join("auth.json");

        // 文件不存在 → 返回空状态（首次启动/未登录）
        if !path.exists() {
            let state = PersistedAuthState::default();
            *self.cache.lock().await = Some(state.clone());
            return Ok(state);
        }

        let raw = std::fs::read_to_string(&path).map_err(|e| format!("读取认证文件失败: {}", e))?;

        let root: Value =
            serde_json::from_str(&raw).map_err(|e| format!("解析认证状态 JSON 失败: {}", e))?;

        let mut state = PersistedAuthState::default();

        // 读取 current_user（敏感字段逐字段 SDK 解密，login_type 明文）
        if let Some(user_obj) = root.get("current_user").and_then(|v| v.as_object()) {
            let login_type = user_obj
                .get("login_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !login_type.is_empty() {
                // 必填字段：解密失败回退空串（与注册表 load 行为一致）
                let name = self.decrypt_field(user_obj, "name").await;
                let uuid = self.decrypt_field(user_obj, "uuid").await;
                let access_token = self.decrypt_field(user_obj, "access_token").await;
                let client_token = self.decrypt_field(user_obj, "client_token").await;

                // 可空字段：解密失败或字段缺失返回 None
                let profile_json = self.decrypt_opt_field(user_obj, "profile_json").await;
                let refresh_token = self.decrypt_opt_field(user_obj, "refresh_token").await;
                let expires_at = match self.decrypt_opt_field(user_obj, "expires_at").await {
                    Some(s) => s.parse::<u64>().ok(),
                    None => None,
                };
                let server_url = self.decrypt_opt_field(user_obj, "server_url").await;
                let server_name = self.decrypt_opt_field(user_obj, "server_name").await;

                state.current_user = Some(CurrentUser {
                    name,
                    uuid,
                    access_token,
                    client_token,
                    login_type,
                    profile_json,
                    refresh_token,
                    expires_at,
                    server_url,
                    server_name,
                });
            }
        }

        // 读取多账号列表（加密 JSON 字符串 → SDK 解密 → 反序列化）
        state.ms_accounts = self.decrypt_account_list(&root, "ms_accounts").await;
        state.offline_accounts = self.decrypt_account_list(&root, "offline_accounts").await;
        state.authlib_accounts = self.decrypt_account_list(&root, "authlib_accounts").await;

        log_info!(
            "[Auth] Loaded persisted auth state: current_user={}, ms_accounts={}, offline_accounts={}, authlib_accounts={}",
            state
                .current_user
                .as_ref()
                .map(|u| u.name.as_str())
                .unwrap_or("none"),
            state.ms_accounts.len(),
            state.offline_accounts.len(),
            state.authlib_accounts.len(),
        );

        // 写入缓存，后续 load 直接返回
        *self.cache.lock().await = Some(state.clone());

        Ok(state)
    }

    /// 解密必填字符串字段（非 Windows）
    ///
    /// 字段缺失或解密失败时返回空字符串（与注册表 load 的 `unwrap_or_default()` 行为一致）。
    #[cfg(not(windows))]
    async fn decrypt_field(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
        name: &str,
    ) -> String {
        match obj.get(name).and_then(|v| v.as_str()) {
            Some(cipher) => self.decrypt(cipher).await.unwrap_or_default(),
            None => String::new(),
        }
    }

    /// 解密可空字符串字段（非 Windows）
    ///
    /// 字段为 null/缺失时返回 None；字段存在则 SDK 解密，解密失败也返回 None。
    #[cfg(not(windows))]
    async fn decrypt_opt_field(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
        name: &str,
    ) -> Option<String> {
        match obj.get(name).and_then(|v| v.as_str()) {
            Some(cipher) => self.decrypt(cipher).await.ok(),
            None => None,
        }
    }

    /// 解密多账号列表字段（非 Windows）
    ///
    /// 字段为 null/缺失或解密失败或反序列化失败时返回空 Vec。
    #[cfg(not(windows))]
    async fn decrypt_account_list<T>(&self, root: &serde_json::Value, name: &str) -> Vec<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let Some(cipher) = root.get(name).and_then(|v| v.as_str()) else {
            return Vec::new();
        };
        let json = match self.decrypt(cipher).await {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        if json.is_empty() {
            return Vec::new();
        }
        serde_json::from_str(&json).unwrap_or_default()
    }

    // ============================================================
    // Windows：注册表逐字段 SDK 解密读取
    // ============================================================

    /// 从注册表加载认证状态（Windows）
    ///
    /// 读取注册表 `HKCU\Software\MoLaunch` 逐字段（敏感字段 SDK 解密），
    /// 构造 `PersistedAuthState`。注册表键不存在时返回 `PersistedAuthState::default()`。
    #[cfg(windows)]
    async fn load_from_registry(&self) -> Result<PersistedAuthState, String> {
        use crate::storage::registry::{reg_get, reg_key};

        use super::registry::{
            KEY_AUTHLIB_ACCOUNTS, KEY_AUTHLIB_CURRENT_ACCESS, KEY_AUTHLIB_CURRENT_CLIENT,
            KEY_AUTHLIB_CURRENT_NAME, KEY_AUTHLIB_CURRENT_SERVER_NAME,
            KEY_AUTHLIB_CURRENT_SERVER_URL, KEY_AUTHLIB_CURRENT_UUID, KEY_LEGACY_NAME,
            KEY_LEGACY_UUID, KEY_LOGIN_TYPE, KEY_MS_ACCOUNTS, KEY_MS_CURRENT_ACCESS,
            KEY_MS_CURRENT_EXPIRES, KEY_MS_CURRENT_NAME, KEY_MS_CURRENT_PROFILE,
            KEY_MS_CURRENT_REFRESH, KEY_MS_CURRENT_UUID, KEY_OFFLINE_ACCOUNTS,
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
