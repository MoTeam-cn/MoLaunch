//! 认证持久化模块（Windows 注册表 `HKCU\Software\MoLaunch`）
//!
//! 每字段单独存为注册表键值：敏感字段（Token/用户名/UUID）用 SDK DES 加密，
//! 非敏感字段（登录类型）明文；多账号列表用一个加密 JSON 字符串存储。
//! 子模块：types（数据结构）/ registry（低层 reg_* 自由函数 + 键名常量）/
//! operations（11 个高层方法）/ load（注册表读取）/ save（注册表写入）。

mod load;
mod operations;
mod registry;
mod save;
mod types;

pub use types::{
    CurrentUser, PersistedAuthState, StoredAuthlibAccount, StoredMsAccount, StoredOfflineAccount,
};

use crate::sdk::SdkInstance;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use crate::storage::registry::{reg_get, reg_set};

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
    // 缓存控制
    // --------------------------------------------------------

    /// 清除内存缓存，强制下次 load 从注册表重新读取
    pub async fn invalidate(&self) {
        *self.cache.lock().await = None;
    }
}
