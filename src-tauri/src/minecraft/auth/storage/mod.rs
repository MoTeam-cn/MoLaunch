//! 认证持久化模块（跨平台文件存储）
//!
//! 存储路径：Windows `%APPDATA%/.MolaLaunch/auth.json`，macOS/Linux `~/.config/MolaLaunch/auth.json`。
//! 整个 `PersistedAuthState` 序列化为 JSON 后用 SDK DES 加密，写入单文件；Unix 显式设置 0o600 权限。
//! 子模块：types（数据结构）/ save（文件写入）/ load（文件读取）/ operations（11 个高层方法）。
//!
//! 历史设计：v0.1.0-beta.1 之前使用 Windows 注册表 `HKCU\Software\MoLaunch` 逐字段存储，
//! 非 Windows 平台为 stub（save 静默 Ok(())、load 返回 default）。现改为跨平台文件存储，
//! 老用户升级后需要重新登录（beta 阶段允许，未做注册表→文件迁移）。

mod load;
mod operations;
mod save;
mod types;

pub use types::{
    CurrentUser, PersistedAuthState, StoredAuthlibAccount, StoredMsAccount, StoredOfflineAccount,
};

use crate::sdk::SdkInstance;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// ============================================================
// 认证存储管理器
// ============================================================

/// 认证存储文件在 AppData 目录下的相对路径
const AUTH_FILE: &str = "auth.json";

/// 认证存储管理器
///
/// 使用跨平台文件存储认证信息。整个 `PersistedAuthState` 序列化为 JSON 后用 SDK DES 加密，
/// 写入单文件（Windows `%APPDATA%/.MolaLaunch/auth.json`，Unix `~/.config/MolaLaunch/auth.json`）。
/// Unix 显式设置 0o600 权限保护敏感字段。
pub struct AuthStorage {
    /// SDK 实例引用（用于 DES 加解密）
    sdk: Arc<TokioMutex<Option<SdkInstance>>>,
    /// 内存缓存（避免每次命令都重新读文件+解密+打日志）
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

    // --------------------------------------------------------
    // 文件存储路径
    // --------------------------------------------------------

    /// 解析认证存储文件路径
    ///
    /// - Windows: `%APPDATA%/.MolaLaunch/auth.json`
    /// - macOS/Linux: `~/.config/MolaLaunch/auth.json`
    ///
    /// 路径解析复用 `crate::storage::appdata::appdata_root`，与 online/device.json、
    /// certs、providers 等全局共享资源保持一致的目录约定。
    /// 父目录不自动创建（由调用方按需 `create_dir_all`）。环境变量缺失时返回 Err。
    fn storage_path() -> Result<PathBuf, String> {
        Ok(crate::storage::appdata::appdata_root()?.join(AUTH_FILE))
    }

    // --------------------------------------------------------
    // 缓存控制
    // --------------------------------------------------------

    /// 清除内存缓存，强制下次 load 从文件重新读取
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
