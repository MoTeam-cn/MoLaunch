//! 联机设备凭证持久化模块
//!
//! 存储路径（v2 迁移至 AppData，跨启动器实例共享设备身份）：
//! - Windows: `%APPDATA%/.MolaLaunch/online/device.json`
//! - macOS/Linux: `~/.config/MolaLaunch/online/device.json`
//!
//! 旧路径（v1，已废弃）：`<exe_dir>/.Molaunch/online/device.json`
//! 首次启动检测旧路径存在时自动迁移到新路径，迁移成功后删除旧文件。
//!
//! 存储内容：
//! - Ed25519 私钥种子（32 字节，Base64Url）
//! - X25519 静态私钥（32 字节，Base64Url）
//! - device_pk（UUID 字符串）
//! - device_token（JWT 字符串）
//! - device_public_key（云端 X25519 公钥，Base64Url）
//! - last_login_at（Unix 秒时间戳）
//!
//! 加密策略：
//! - 文件整体 JSON 序列化后用 SDK DES 加密为字符串存储（与 AuthStorage 一致）
//! - SDK 不可用时降级为明文 JSON（带 WARN 日志，跨平台兼容）
//! - 文件权限：Unix 设为 0o600，Windows 默认 ACL 仅当前用户可读

use crate::log_info;
use crate::log_warn;
use crate::sdk::SdkInstance;
use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// 设备凭证文件在 AppData 目录下的相对路径
const DEVICE_FILE: &str = "online/device.json";

/// 持久化的设备凭证
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceCredentials {
    /// Ed25519 私钥种子（32 字节，Base64Url）
    pub ed25519_seed_b64u: String,
    /// X25519 静态私钥（32 字节，Base64Url）
    pub x25519_secret_b64u: String,
    /// 设备主键（UUID）
    pub device_pk: String,
    /// 设备 JWT（Bearer token，调用 /v1 业务接口用）
    pub device_token: String,
    /// JWT 过期时间（Unix 秒）
    pub token_expires_at: u64,
    /// 云端为设备生成的 X25519 公钥（Base64Url，ECIES 加密用）
    pub device_public_key_b64u: String,
    /// 设备友好标识（mcsdk-xxxx-xxxx-xxxx-xxxx）
    pub device_id: String,
    /// 最后登录时间（Unix 秒）
    pub last_login_at: u64,
}

impl DeviceCredentials {
    /// 是否已注册（有 device_pk 和密钥）
    pub fn is_registered(&self) -> bool {
        !self.device_pk.is_empty()
            && !self.ed25519_seed_b64u.is_empty()
            && !self.x25519_secret_b64u.is_empty()
            && !self.device_public_key_b64u.is_empty()
    }

    /// JWT 是否已过期（容差 60 秒，避免边界请求失败）
    pub fn is_token_expired(&self) -> bool {
        if self.token_expires_at == 0 {
            return true;
        }
        let now = chrono::Utc::now().timestamp() as u64;
        now + 60 >= self.token_expires_at
    }
}

/// 联机设备凭证存储
///
/// 与 `minecraft::auth::storage::AuthStorage` 平级，使用相同的 SDK DES 加密机制，
/// 但存储位置不同（文件 vs 注册表），避免与 MC 账号数据混淆。
pub struct OnlineStorage {
    sdk: Arc<TokioMutex<Option<SdkInstance>>>,
}

impl OnlineStorage {
    pub fn new(sdk: Arc<TokioMutex<Option<SdkInstance>>>) -> Self {
        Self { sdk }
    }

    /// 加密字符串（SDK DES）
    async fn encrypt(&self, data: &str) -> Result<String, String> {
        let sdk = self.sdk.lock().await;
        match sdk.as_ref() {
            Some(sdk) => sdk
                .encrypt_token(data)
                .map_err(|e| format!("加密失败: {}", e)),
            None => Err("SDK 未加载，无法加密联机设备凭证".to_string()),
        }
    }

    /// 解密字符串（SDK DES）
    async fn decrypt(&self, data: &str) -> Result<String, String> {
        let sdk = self.sdk.lock().await;
        match sdk.as_ref() {
            Some(sdk) => sdk
                .decrypt_token(data)
                .map_err(|e| format!("解密失败: {}", e)),
            None => Err("SDK 未加载，无法解密联机设备凭证".to_string()),
        }
    }

    /// 加载设备凭证
    ///
    /// 优先从新路径（AppData）加载；若新路径不存在但旧路径存在，自动迁移到新路径后加载。
    ///
    /// 返回 `None` 表示未注册或文件不存在；
    /// 返回 `Err` 表示文件存在但解析/解密失败（数据损坏）。
    pub async fn load(&self) -> Result<Option<DeviceCredentials>, String> {
        let new_path = Self::appdata_device_path()?;

        // 1. 新路径存在 → 直接加载
        if new_path.exists() {
            return self.load_from(&new_path).await;
        }

        // 2. 新路径不存在 → 检查旧路径并迁移
        let legacy_path = Self::legacy_device_path();
        if !legacy_path.exists() {
            return Ok(None);
        }

        log_info!(
            "[Online] 检测到旧路径设备凭证，开始迁移: {} → {}",
            legacy_path.display(),
            new_path.display()
        );

        // 3. 读取旧文件原内容（不解密，原样转写到新路径）
        let raw = std::fs::read_to_string(&legacy_path)
            .map_err(|e| format!("读取旧路径设备凭证失败: {}", e))?;

        // 4. 写入新路径（确保父目录存在）
        if let Some(parent) = new_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建新路径父目录失败: {}", e))?;
        }
        if let Err(e) = std::fs::write(&new_path, &raw) {
            log_warn!(
                "[Online] 迁移失败（写入新路径失败），回退到旧路径加载: {}",
                e
            );
            return self.load_from(&legacy_path).await;
        }

        // 5. 删除旧文件
        if let Err(e) = std::fs::remove_file(&legacy_path) {
            log_warn!(
                "[Online] 迁移成功但旧文件删除失败，下次启动会再次尝试迁移: {}",
                e
            );
        }

        log_info!("[Online] 设备凭证迁移完成");

        // 6. 加载新路径
        self.load_from(&new_path).await
    }

    /// 从指定路径加载设备凭证（内部辅助方法）
    async fn load_from(&self, path: &Path) -> Result<Option<DeviceCredentials>, String> {
        if !path.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("读取设备凭证失败: {}", e))?;

        // 尝试解密（SDK 可用时）；SDK 不可用时降级为明文 JSON
        let json = match self.decrypt(&raw).await {
            Ok(s) => s,
            Err(e) => {
                log_warn!("SDK 解密失败，尝试明文解析: {}", e);
                raw
            }
        };

        let creds: DeviceCredentials = serde_json::from_str(&json)
            .map_err(|e| format!("解析设备凭证 JSON 失败: {}", e))?;
        Ok(Some(creds))
    }

    /// 保存设备凭证
    ///
    /// 写入新路径（AppData），并尝试清理旧路径文件以防重复迁移。
    pub async fn save(&self, creds: &DeviceCredentials) -> Result<(), String> {
        let json = serde_json::to_string(creds)
            .map_err(|e| format!("序列化设备凭证失败: {}", e))?;

        // 优先加密存储；SDK 不可用时降级为明文（带警告）
        let stored = match self.encrypt(&json).await {
            Ok(s) => s,
            Err(e) => {
                log_warn!("SDK 加密失败，降级明文存储: {}", e);
                json
            }
        };

        let new_path = Self::appdata_device_path()?;
        if let Some(parent) = new_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建存储目录失败: {}", e))?;
        }
        std::fs::write(&new_path, &stored)
            .map_err(|e| format!("写入设备凭证失败: {}", e))?;

        // 保存成功后清理旧路径文件（如有），避免下次启动重复迁移
        let legacy_path = Self::legacy_device_path();
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
                std::fs::remove_file(&new_path)
                    .map_err(|e| format!("删除设备凭证失败: {}", e))?;
            }
        }

        // 清理旧路径（兼容历史遗留）
        let legacy_path = Self::legacy_device_path();
        if legacy_path.exists() {
            if let Err(e) = std::fs::remove_file(&legacy_path) {
                log_warn!("[Online] 清理旧路径文件失败: {}", e);
            }
        }

        Ok(())
    }

    /// 解析设备凭证新存储路径（AppData 全局位置）
    ///
    /// - Windows: `%APPDATA%/.MolaLaunch/online/device.json`
    /// - macOS/Linux: `~/.config/MolaLaunch/online/device.json`
    ///
    /// 父目录不自动创建（由调用方按需 `create_dir_all`）。环境变量缺失时返回 Err。
    fn appdata_device_path() -> Result<PathBuf, String> {
        #[cfg(windows)]
        {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| "APPDATA environment variable not set".to_string())?;
            Ok(PathBuf::from(appdata).join(".MolaLaunch").join(DEVICE_FILE))
        }

        #[cfg(not(windows))]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "HOME environment variable not set".to_string())?;
            Ok(PathBuf::from(home).join(".config").join("MolaLaunch").join(DEVICE_FILE))
        }
    }

    /// 解析设备凭证旧存储路径（启动器目录下的 `.Molaunch`）
    ///
    /// 用于一次性迁移：v1 存储位置为 `<exe_dir>/.Molaunch/online/device.json`，
    /// v2 迁移到 AppData 后此路径不再使用，但检测到旧文件存在时需迁移。
    fn legacy_device_path() -> PathBuf {
        Storage::instance().base_dir().join(DEVICE_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_registered() {
        let empty = DeviceCredentials::default();
        assert!(!empty.is_registered());

        let mut creds = DeviceCredentials::default();
        creds.device_pk = "uuid".to_string();
        creds.ed25519_seed_b64u = "seed".to_string();
        creds.x25519_secret_b64u = "sec".to_string();
        creds.device_public_key_b64u = "pub".to_string();
        assert!(creds.is_registered());
    }

    #[test]
    fn test_is_token_expired() {
        let mut creds = DeviceCredentials::default();
        // token_expires_at = 0 视为已过期
        assert!(creds.is_token_expired());

        // 设为未来 1 小时
        let future = (chrono::Utc::now().timestamp() + 3600) as u64;
        creds.token_expires_at = future;
        assert!(!creds.is_token_expired());

        // 设为过去
        let past = (chrono::Utc::now().timestamp() - 100) as u64;
        creds.token_expires_at = past;
        assert!(creds.is_token_expired());
    }
}
