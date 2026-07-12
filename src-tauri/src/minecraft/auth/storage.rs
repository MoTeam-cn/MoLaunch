//! 认证持久化模块
//!
//! 负责将微软登录的 Token 安全存储到磁盘，支持：
//! - 多账号管理（以用户名为键）
//! - Token 加密（使用 SDK DES 加密，密钥为 mcsdk-{设备码}）
//! - 会话恢复（应用重启后自动加载已存储的登录状态）
//! - 自动静默刷新（Token 过期时使用 Refresh Token 刷新）
//!
//! 存储文件：.Molaunch/auth.json（加密后的 JSON）

use crate::log_info;
use crate::sdk::SdkInstance;
use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use super::microsoft::MicrosoftLoginResult;

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

/// 持久化的认证状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedAuthState {
    /// 当前登录的账号（离线或微软）
    pub current_user: Option<CurrentUser>,
    /// 已保存的微软账号列表（多账号）
    pub ms_accounts: Vec<StoredMsAccount>,
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

/// 认证存储管理器
pub struct AuthStorage {
    /// SDK 实例引用（用于 DES 加解密）
    sdk: Arc<TokioMutex<Option<SdkInstance>>>,
}

impl AuthStorage {
    pub fn new(sdk: Arc<TokioMutex<Option<SdkInstance>>>) -> Self {
        Self { sdk }
    }

    /// 认证文件路径
    fn auth_file_path() -> std::path::PathBuf {
        Storage::instance().base_dir().join("auth.json")
    }

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

    /// 加载持久化的认证状态
    pub async fn load(&self) -> Result<PersistedAuthState, String> {
        let path = Self::auth_file_path();
        if !path.exists() {
            return Ok(PersistedAuthState::default());
        }

        let encrypted =
            std::fs::read_to_string(&path).map_err(|e| format!("读取认证文件失败: {}", e))?;
        if encrypted.is_empty() {
            return Ok(PersistedAuthState::default());
        }

        let decrypted = self.decrypt(&encrypted).await?;
        let state: PersistedAuthState =
            serde_json::from_str(&decrypted).map_err(|e| format!("解析认证状态失败: {}", e))?;

        log_info!(
            "Loaded persisted auth state: current_user={}, ms_accounts={}",
            state
                .current_user
                .as_ref()
                .map(|u| u.name.as_str())
                .unwrap_or("none"),
            state.ms_accounts.len()
        );

        Ok(state)
    }

    /// 保存认证状态到磁盘
    pub async fn save(&self, state: &PersistedAuthState) -> Result<(), String> {
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| format!("序列化认证状态失败: {}", e))?;
        let encrypted = self.encrypt(&json).await?;

        let path = Self::auth_file_path();

        // 原子写入：先写 .tmp 再 rename
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, &encrypted).map_err(|e| format!("写入认证文件失败: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&tmp) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&tmp, perms);
            }
        }

        std::fs::rename(&tmp, &path).map_err(|e| format!("重命名认证文件失败: {}", e))?;

        Ok(())
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
            client_token: String::new(), // 微软登录无 client_token
            login_type: "Microsoft".to_string(),
            profile_json: Some(result.profile_json.clone()),
            refresh_token: Some(result.refresh_token.clone()),
            expires_at: Some(result.expires_at),
        });

        self.save(&state).await
    }

    /// 保存离线登录并设为当前用户
    pub async fn save_offline_login(&self, username: &str, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

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
