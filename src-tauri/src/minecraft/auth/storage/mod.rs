//! 认证持久化模块（双轨制存储）
//!
//! - **Windows**：注册表 `HKCU\Software\MoLaunch` 逐字段存储。敏感字段（Token/用户名/UUID）
//!   用 SDK DES 加密后写入独立键值，非敏感字段（登录类型）明文；多账号列表序列化为
//!   JSON 字符串后整体 SDK 加密写入单键。
//! - **非 Windows**：JSON 文件 `%APPDATA%/.Molaunch/auth.json`（macOS/Linux 为
//!   `~/.config/Molaunch/auth.json`）结构化逐字段加密。明文 JSON 结构中每个敏感字段
//!   单独 SDK 加密为字符串值，非敏感字段（login_type）明文；多账号列表先序列化为 JSON
//!   字符串再 SDK 加密。Unix 显式设置 0o600 权限保护敏感字段。
//!
//! 子模块：types（数据结构）/ registry（注册表键名常量，仅 Windows 使用）/
//! operations（11 个高层方法）/ load（读取）/ save（写入）。
//!
//! 历史设计：v0.1.0-beta.1 之前 Windows 用注册表、非 Windows 为 stub。
//! v0.1.0-beta.1 曾改为跨平台整体加密文件存储，现回归双轨制：Windows 恢复注册表，
//! 非 Windows 改为结构化逐字段加密文件（避免整体加密一个 JSON 字符串）。

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
