//! 联机设备凭证持久化模块
//! 存储路径：Windows `%APPDATA%/.Molaunch/online/device.json`，macOS/Linux `~/.config/Molaunch/online/device.json`。
//! 旧路径（v1 已废弃）`<exe_dir>/.Molaunch/online/device.json` 启动时由 `migrations::online_legacy` 自动迁移。
//! 加密策略：文件整体 JSON 序列化后用 SDK DES 加密；SDK 不可用时降级明文（带 WARN）。

use crate::log_warn;
use crate::migrations::online_legacy::legacy_device_path;
use crate::sdk::SdkInstance;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// 持久化的设备凭证
///
/// 安全约束（方案 C）：仅派生 `Deserialize`（从加密文件反序列化），**不派生 `Serialize`**。
/// 含 Ed25519 私钥种子、X25519 私钥、device_pk、device_token 等高敏感字段，
/// 持久化写入时调用 `to_storage_json()`；IPC 返回前端用 `DeviceStatus`（已 `#[serde(skip)]` device_pk）。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DeviceCredentials {
    /// Ed25519 私钥种子（32 字节，Base64Url）
    pub ed25519_seed_b64u: String,
    /// X25519 静态私钥（32 字节，Base64Url）
    pub x25519_secret_b64u: String,
    /// 设备主键（UUID）
    pub device_pk: String,
    /// 设备 access JWT（Bearer token，调用 /v1 业务接口用，1h 有效期）
    pub device_token: String,
    /// access JWT 过期时间（Unix 秒）
    pub token_expires_at: u64,
    /// refresh_token（用于续期 access token，30d 有效期；空串表示旧版凭证未持有）
    #[serde(default)]
    pub refresh_token: String,
    /// refresh_token 过期时间（Unix 秒；0 表示未设置/旧版凭证）
    #[serde(default)]
    pub refresh_expires_at: u64,
    /// 云端为设备生成的 X25519 公钥（Base64Url，ECIES 加密用）
    pub device_public_key_b64u: String,
    /// 设备友好标识（mcsdk-xxxx-xxxx-xxxx-xxxx）
    pub device_id: String,
    /// 最后登录时间（Unix 秒）
    pub last_login_at: u64,
    /// 签发此凭证的 API 服务端地址（用于检测 api_server_url 切换后旧凭证失效）
    #[serde(default)]
    pub api_server_url: String,
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

    /// refresh_token 是否已过期（容差 60 秒）
    ///
    /// `refresh_expires_at == 0` 视为已过期（旧版凭证未设置或未持有 refresh_token）。
    pub fn is_refresh_token_expired(&self) -> bool {
        if self.refresh_expires_at == 0 {
            return true;
        }
        let now = chrono::Utc::now().timestamp() as u64;
        now + 60 >= self.refresh_expires_at
    }

    /// 构建包含全部字段（含私钥/JWT）的 JSON，仅供持久化写入加密文件使用
    pub fn to_storage_json(&self) -> serde_json::Value {
        serde_json::json!({
            "ed25519_seed_b64u": self.ed25519_seed_b64u,
            "x25519_secret_b64u": self.x25519_secret_b64u,
            "device_pk": self.device_pk,
            "device_token": self.device_token,
            "token_expires_at": self.token_expires_at,
            "refresh_token": self.refresh_token,
            "refresh_expires_at": self.refresh_expires_at,
            "device_public_key_b64u": self.device_public_key_b64u,
            "device_id": self.device_id,
            "last_login_at": self.last_login_at,
            "api_server_url": self.api_server_url,
        })
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

        // 尝试解密（SDK 可用时）；SDK 不可用时降级为明文 JSON
        let json = match self.decrypt(&raw).await {
            Ok(s) => s,
            Err(e) => {
                log_warn!("SDK 解密失败，尝试明文解析: {}", e);
                raw
            }
        };

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
