//! 认证持久化模块：Windows 注册表 / 非 Windows JSON 文件双轨制存储（敏感字段 SDK 加密）

mod load;
mod operations;
#[cfg(windows)]
mod registry;
mod save;
mod types;

pub use types::{
    CurrentUser, PersistedAuthState, StoredAuthlibAccount, StoredMsAccount, StoredOfflineAccount,
};

use crate::sdk::SdkInstance;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[cfg(unix)]
use crate::log_warn;

// 认证存储管理器

/// 认证存储管理器
///
/// 双轨制存储：
/// - Windows：注册表 `HKCU\Software\MoLaunch` 逐字段 SDK 加密存储。
/// - 非 Windows：JSON 文件结构化逐字段 SDK 加密存储，Unix 设置 0o600 权限。
///
/// `load`/`save` 内部按平台分支，`operations` 仅依赖这两个方法，与存储细节解耦。
pub struct AuthStorage {
    /// SDK 实例引用（用于 DES 加解密）
    sdk: Arc<TokioMutex<Option<SdkInstance>>>,
    /// 内存缓存（避免每次命令都重新读存储+解密+打日志）
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

    // 加解密工具

    /// 加密数据（使用 SDK 内置的 DES 加密）
    async fn encrypt(&self, data: &str) -> Result<String, String> {
        crate::utils::sdk_crypto::encrypt_with_sdk(&self.sdk, data, "认证数据").await
    }

    /// 解密数据（使用 SDK 内置的 DES 解密）
    async fn decrypt(&self, data: &str) -> Result<String, String> {
        crate::utils::sdk_crypto::decrypt_with_sdk(&self.sdk, data, "认证数据").await
    }

    /// 加密并写入注册表（仅 Windows）
    ///
    /// 等价于旧版 `reg_set_encrypted`：self.encrypt SDK DES 加密 → reg_set 写入注册表。
    /// 非 Windows 平台不编译此方法（注册表不可用）。
    #[cfg(windows)]
    async fn reg_set_encrypted(
        &self,
        key: &winreg::RegKey,
        name: &str,
        value: &str,
    ) -> Result<(), String> {
        use crate::storage::registry::reg_set;

        let encrypted = self.encrypt(value).await?;
        reg_set(key, name, &encrypted)
    }

    /// 读取并解密注册表（仅 Windows）
    ///
    /// 等价于旧版 `reg_get_decrypted`：reg_get 读取明文值 → self.decrypt SDK DES 解密。
    /// 注册表键不存在或解密失败时返回 None。非 Windows 平台不编译此方法。
    #[cfg(windows)]
    async fn reg_get_decrypted(&self, key: &winreg::RegKey, name: &str) -> Option<String> {
        use crate::storage::registry::reg_get;

        let encrypted = reg_get(key, name)?;
        self.decrypt(&encrypted).await.ok()
    }

    // 缓存控制

    /// 清除内存缓存，强制下次 load 从存储重新读取
    pub async fn invalidate(&self) {
        *self.cache.lock().await = None;
    }
}

/// Unix 下显式设置文件权限为 0o600（仅当前用户可读写），防止其他用户读取 token
///
/// Windows 依赖 NTFS 默认 ACL（继承父目录权限，通常已足够），无需显式设置。
#[cfg(unix)]
fn restrict_file_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
        log_warn!("[Auth] 设置认证文件权限 0o600 失败: {}", e);
    }
}
