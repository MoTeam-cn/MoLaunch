//! `AuthStorage::save` 实现：注册表 / JSON 文件双轨制写入（敏感字段 SDK 加密，写后刷新缓存）

use super::types::PersistedAuthState;
use super::AuthStorage;

impl AuthStorage {
    /// 保存认证状态
    ///
    /// Windows 写入注册表（逐字段 SDK 加密）；非 Windows 写入 JSON 文件
    /// （结构化逐字段加密）。写完后刷新内存缓存。
    pub async fn save(&self, state: &PersistedAuthState) -> Result<(), String> {
        #[cfg(not(windows))]
        {
            self.save_to_file(state).await
        }

        #[cfg(windows)]
        {
            self.save_to_registry(state).await
        }
    }

    // ============================================================
    // 非 Windows：JSON 文件结构化逐字段加密存储
    // ============================================================

    /// 保存认证状态到 JSON 文件（非 Windows）
    ///
    /// 存储格式：明文 JSON 结构，每个敏感字段单独 SDK 加密为字符串值，非敏感字段
    /// （login_type）明文；多账号列表先序列化为 JSON 字符串再 SDK 加密。
    /// Unix 显式设置 0o600 权限保护敏感字段；写完后刷新内存缓存。
    #[cfg(not(windows))]
    async fn save_to_file(&self, state: &PersistedAuthState) -> Result<(), String> {
        use serde_json::{json, Map, Value};

        let path = crate::storage::appdata::appdata_root()?.join("auth.json");

        // 构造 current_user 对象（敏感字段逐字段加密，login_type 明文）
        let current_user: Value = match &state.current_user {
            Some(user) => {
                let mut obj = Map::new();
                obj.insert("name".into(), json!(self.encrypt(&user.name).await?));
                obj.insert("uuid".into(), json!(self.encrypt(&user.uuid).await?));
                obj.insert(
                    "access_token".into(),
                    json!(self.encrypt(&user.access_token).await?),
                );
                obj.insert(
                    "client_token".into(),
                    json!(self.encrypt(&user.client_token).await?),
                );
                // login_type 非敏感，明文存储
                obj.insert("login_type".into(), json!(user.login_type));
                // 可空字段：Some 加密，None 存 null
                obj.insert(
                    "profile_json".into(),
                    match &user.profile_json {
                        Some(v) => json!(self.encrypt(v).await?),
                        None => Value::Null,
                    },
                );
                obj.insert(
                    "refresh_token".into(),
                    match &user.refresh_token {
                        Some(v) => json!(self.encrypt(v).await?),
                        None => Value::Null,
                    },
                );
                obj.insert(
                    "expires_at".into(),
                    match user.expires_at {
                        Some(v) => json!(self.encrypt(&v.to_string()).await?),
                        None => Value::Null,
                    },
                );
                obj.insert(
                    "server_url".into(),
                    match &user.server_url {
                        Some(v) => json!(self.encrypt(v).await?),
                        None => Value::Null,
                    },
                );
                obj.insert(
                    "server_name".into(),
                    match &user.server_name {
                        Some(v) => json!(self.encrypt(v).await?),
                        None => Value::Null,
                    },
                );
                Value::Object(obj)
            }
            None => Value::Null,
        };

        // 多账号列表：先序列化为 JSON 字符串再 SDK 加密
        // ms_accounts / authlib_accounts 通过 to_storage_json 手动序列化，
        // 避免派生 Serialize 误暴露 token/password 到 IPC
        let ms_accounts_cipher = if state.ms_accounts.is_empty() {
            Value::Null
        } else {
            let arr: Vec<Value> = state
                .ms_accounts
                .iter()
                .map(|a| a.to_storage_json())
                .collect();
            let json_str = serde_json::to_string(&arr)
                .map_err(|e| format!("序列化微软账号列表失败: {}", e))?;
            json!(self.encrypt(&json_str).await?)
        };

        let offline_accounts_cipher = if state.offline_accounts.is_empty() {
            Value::Null
        } else {
            // 离线账号无敏感字段，可直接序列化；统一加密保持存储格式一致
            let json_str = serde_json::to_string(&state.offline_accounts)
                .map_err(|e| format!("序列化离线账号列表失败: {}", e))?;
            json!(self.encrypt(&json_str).await?)
        };

        let authlib_accounts_cipher = if state.authlib_accounts.is_empty() {
            Value::Null
        } else {
            let arr: Vec<Value> = state
                .authlib_accounts
                .iter()
                .map(|a| a.to_storage_json())
                .collect();
            let json_str = serde_json::to_string(&arr)
                .map_err(|e| format!("序列化 authlib 账号列表失败: {}", e))?;
            json!(self.encrypt(&json_str).await?)
        };

        // 顶层 login_type 非敏感，明文存储（便于人工排查/快速判断登录类型）
        let root = json!({
            "login_type": state.current_user.as_ref().map(|u| u.login_type.as_str()).unwrap_or(""),
            "current_user": current_user,
            "ms_accounts": ms_accounts_cipher,
            "offline_accounts": offline_accounts_cipher,
            "authlib_accounts": authlib_accounts_cipher,
        });

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建认证存储目录失败: {}", e))?;
        }

        let content =
            serde_json::to_string(&root).map_err(|e| format!("序列化认证状态失败: {}", e))?;
        std::fs::write(&path, &content).map_err(|e| format!("写入认证文件失败: {}", e))?;

        // Unix 下显式设置文件权限为 0o600（仅当前用户可读写），防止其他用户读取 token
        #[cfg(unix)]
        restrict_file_permissions(&path);

        // 更新内存缓存
        *self.cache.lock().await = Some(state.clone());

        Ok(())
    }

    // ============================================================
    // Windows：注册表逐字段 SDK 加密存储
    // ============================================================

    /// 保存认证状态到注册表（Windows）
    ///
    /// 将 `PersistedAuthState` 的所有字段写入注册表 `HKCU\Software\MoLaunch`。
    /// 先清除所有旧值，再写入新值，确保数据一致；写完后刷新内存缓存。
    #[cfg(windows)]
    async fn save_to_registry(&self, state: &PersistedAuthState) -> Result<(), String> {
        use crate::storage::registry::{reg_delete, reg_key, reg_set};

        use super::registry::{
            ALL_KEYS, KEY_AUTHLIB_ACCOUNTS, KEY_AUTHLIB_CURRENT_ACCESS, KEY_AUTHLIB_CURRENT_CLIENT,
            KEY_AUTHLIB_CURRENT_NAME, KEY_AUTHLIB_CURRENT_SERVER_NAME,
            KEY_AUTHLIB_CURRENT_SERVER_URL, KEY_AUTHLIB_CURRENT_UUID, KEY_LEGACY_NAME,
            KEY_LEGACY_UUID, KEY_LOGIN_TYPE, KEY_MS_ACCOUNTS, KEY_MS_CURRENT_ACCESS,
            KEY_MS_CURRENT_EXPIRES, KEY_MS_CURRENT_NAME, KEY_MS_CURRENT_PROFILE,
            KEY_MS_CURRENT_REFRESH, KEY_MS_CURRENT_UUID, KEY_OFFLINE_ACCOUNTS,
        };

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
                        self.reg_set_encrypted(&key, KEY_MS_CURRENT_EXPIRES, &expires.to_string())
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
                        self.reg_set_encrypted(&key, KEY_AUTHLIB_CURRENT_SERVER_URL, server_url)
                            .await?;
                    }
                    if let Some(ref server_name) = user.server_name {
                        self.reg_set_encrypted(&key, KEY_AUTHLIB_CURRENT_SERVER_NAME, server_name)
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
            let json =
                serde_json::to_string(&arr).map_err(|e| format!("序列化账号列表失败: {}", e))?;
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

/// Unix 下显式设置文件权限为 0o600（仅当前用户可读写），防止其他用户读取 token
///
/// Windows 依赖 NTFS 默认 ACL（继承父目录权限，通常已足够），无需显式设置。
#[cfg(unix)]
fn restrict_file_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
        crate::log_warn!("[Auth] 设置认证文件权限 0o600 失败: {}", e);
    }
}
