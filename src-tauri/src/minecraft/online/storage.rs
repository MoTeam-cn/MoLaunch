//! 联机设备凭证持久化模块
//! 存储路径：Windows `%APPDATA%/.Molaunch/online/device.json`，macOS/Linux `~/.config/Molaunch/online/device.json`。
//! 旧路径（v1 已废弃）`<exe_dir>/.Molaunch/online/device.json` 启动时由 `migrations::online_legacy` 自动迁移。
//! 加密策略：文件整体 JSON 序列化后用 SDK 加密（AES-256-CBC），解密自动兼容旧 DES。

use crate::log_warn;
use crate::migrations::online_legacy::legacy_device_path;
use crate::sdk::SdkInstance;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// 设备凭证结构由私有协议库维护（字段与协议一一对应），此处仅转发。
pub use molaunch_protocol::credentials::DeviceCredentials;

/// 联机设备凭证存储
///
/// 与 `minecraft::auth::storage::AuthStorage` 平级，使用相同的 SDK 加密机制，
/// 但存储位置不同（文件 vs 注册表），避免与 MC 账号数据混淆。
pub struct OnlineStorage {
    sdk: Arc<TokioMutex<Option<SdkInstance>>>,
}

impl OnlineStorage {
    pub fn new(sdk: Arc<TokioMutex<Option<SdkInstance>>>) -> Self {
        Self { sdk }
    }

    /// 加密字符串（SDK AES-256-CBC）
    async fn encrypt(&self, data: &str) -> Result<String, String> {
        crate::utils::sdk_crypto::encrypt_with_sdk(&self.sdk, data, "联机设备凭证").await
    }

    /// 解密字符串（SDK 解密，自动兼容旧 DES）
    async fn decrypt(&self, data: &str) -> Result<String, String> {
        crate::utils::sdk_crypto::decrypt_with_sdk(&self.sdk, data, "联机设备凭证").await
    }

    /// 加载设备凭证
    ///
    /// 从新路径（AppData）加载；旧路径迁移已由 `crate::migrations::online_legacy`
    /// 在启动时执行，此处不再检测旧路径。
    ///
    /// 返回 `None` 表示未注册或文件不存在；
    /// 返回 `Err` 表示文件存在但解析/解密失败（数据损坏）。
    pub async fn load(&self) -> Result<Option<DeviceCredentials>, String> {
        let new_path = Self::appdata_device_path()?;
        if new_path.exists() {
            return self.load_from(&new_path).await;
        }
        Ok(None)
    }

    /// 从指定路径加载设备凭证（内部辅助方法）
    async fn load_from(&self, path: &Path) -> Result<Option<DeviceCredentials>, String> {
        if !path.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(path).map_err(|e| format!("读取设备凭证失败: {}", e))?;
        let json = self.decrypt(&raw).await?;

        let creds: DeviceCredentials =
            serde_json::from_str(&json).map_err(|e| format!("解析设备凭证 JSON 失败: {}", e))?;
        Ok(Some(creds))
    }

    /// 保存设备凭证
    ///
    /// 写入新路径（AppData），并尝试清理旧路径文件以防重复迁移。
    /// 通过 `to_storage_json()` 手动序列化，避免派生 `Serialize` 误将私钥/JWT 暴露到 IPC。
    pub async fn save(&self, creds: &DeviceCredentials) -> Result<(), String> {
        let json = serde_json::to_string(&creds.to_storage_json())
            .map_err(|e| format!("序列化设备凭证失败: {}", e))?;

        let stored = self.encrypt(&json).await?;

        let new_path = Self::appdata_device_path()?;
        if let Some(parent) = new_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建存储目录失败: {}", e))?;
        }
        std::fs::write(&new_path, &stored).map_err(|e| format!("写入设备凭证失败: {}", e))?;

        // Unix 下显式设置文件权限为 0o600（仅当前用户可读写），防止其他用户读取私钥/token
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) =
                std::fs::set_permissions(&new_path, std::fs::Permissions::from_mode(0o600))
            {
                log_warn!("[Online] 设置设备凭证文件权限 0o600 失败: {}", e);
            }
        }

        // 保存成功后清理旧路径文件（如有），避免下次启动重复迁移
        let legacy_path = legacy_device_path();
        if legacy_path.exists() {
            if let Err(e) = std::fs::remove_file(&legacy_path) {
                log_warn!("[Online] 清理旧路径文件失败: {}", e);
            }
        }

        Ok(())
    }

    /// 清除设备凭证（注销设备时调用）
    ///
    /// 同时删除新路径和旧路径文件，确保完全清除。
    pub fn clear() -> Result<(), String> {
        // 清理新路径
        if let Ok(new_path) = Self::appdata_device_path() {
            if new_path.exists() {
                std::fs::remove_file(&new_path).map_err(|e| format!("删除设备凭证失败: {}", e))?;
            }
        }

        // 清理旧路径（兼容历史遗留）
        let legacy_path = legacy_device_path();
        if legacy_path.exists() {
            if let Err(e) = std::fs::remove_file(&legacy_path) {
                log_warn!("[Online] 清理旧路径文件失败: {}", e);
            }
        }

        Ok(())
    }

    /// 解析设备凭证新存储路径（AppData 全局位置）
    ///
    /// - Windows: `%APPDATA%/.Molaunch/online/device.json`
    /// - macOS/Linux: `~/.config/Molaunch/online/device.json`
    ///
    /// 路径解析复用 `crate::storage::appdata::appdata_subdir`，与 certs/providers/auth 等
    /// 全局共享资源保持一致的目录约定。父目录不自动创建（由调用方按需 `create_dir_all`）。
    /// 环境变量缺失时返回 Err。
    fn appdata_device_path() -> Result<PathBuf, String> {
        Ok(crate::storage::appdata::appdata_subdir("online")?.join("device.json"))
    }
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
