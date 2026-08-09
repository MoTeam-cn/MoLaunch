//! FRP 厂商认证 token 存储：文件 + SDK 加密（全局共享设备级数据）

use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex as TokioMutex;

use crate::sdk::SdkInstance;

/// SDK 引用（启动时注入，供 token 加解密）
static SDK_REF: OnceLock<Arc<TokioMutex<Option<SdkInstance>>>> = OnceLock::new();

/// 注入 SDK 引用（lib.rs 启动时调用）
pub fn set_sdk(sdk: Arc<TokioMutex<Option<SdkInstance>>>) {
    let _ = SDK_REF.set(sdk);
}

/// 厂商 token 记录（单文件整体存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct TokenRecord {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// token 过期时间（Unix 秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// 加密字符串（SDK AES-256-CBC）
async fn encrypt(data: &str) -> Result<String, String> {
    let sdk_arc = SDK_REF
        .get()
        .ok_or_else(|| "SDK 未注入，无法加密 token".to_string())?;
    crate::utils::sdk_crypto::encrypt_with_sdk(sdk_arc, data, "FRP token").await
}

/// 解密字符串（SDK 解密，自动兼容旧 DES）；失败时视为无 token
async fn decrypt(data: &str) -> Option<String> {
    let sdk_arc = match SDK_REF.get() {
        Some(arc) => arc.clone(),
        None => {
            crate::log_warn!("[Frp Auth] SDK 未注入，无法解密 token");
            return None;
        }
    };
    crate::utils::sdk_crypto::decrypt_with_sdk_optional(&sdk_arc, data, "FRP token").await
}

/// 存储完整 token 信息（OAuth2 / Device Code 认证成功后调用）
///
/// 整份 TokenRecord 序列化后经 SDK 加密写入 `{provider_id}.json`。
/// `expires_in` 为相对秒数，内部换算为绝对过期时间存储。
pub(super) async fn store_token_info(
    provider_id: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in: Option<u64>,
    scopes: Option<&Vec<String>>,
) -> Result<(), String> {
    super::super::paths::validate_provider_id(provider_id)?;

    let record = TokenRecord {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.map(|s| s.to_string()),
        expires_at: expires_in.map(|secs| now_secs() + secs),
        scopes: scopes.cloned(),
    };
    let json = serde_json::to_string(&record).map_err(|e| format!("序列化 token 失败: {}", e))?;
    let encrypted = encrypt(&json).await?;

    let path = super::super::paths::auth_file_path(provider_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建 auth 目录失败: {}", e))?;
    }
    std::fs::write(&path, &encrypted).map_err(|e| format!("写入 token 文件失败: {}", e))?;

    // Unix 下限制为仅当前用户可读写
    #[cfg(unix)]
    {
        use crate::log_warn;
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)) {
            log_warn!("[Frp Auth] 设置 token 文件权限 0o600 失败: {}", e);
        }
    }

    Ok(())
}

/// 读取厂商 token 记录（不存在或解密失败返回 None）
pub(super) async fn load_token_record(provider_id: &str) -> Result<Option<TokenRecord>, String> {
    super::super::paths::validate_provider_id(provider_id)?;

    let path = super::super::paths::auth_file_path(provider_id);
    if !path.exists() {
        return Ok(None);
    }
    let encrypted =
        std::fs::read_to_string(&path).map_err(|e| format!("读取 token 文件失败: {}", e))?;

    let Some(json) = decrypt(&encrypted).await else {
        return Ok(None);
    };
    let record: TokenRecord =
        serde_json::from_str(&json).map_err(|e| format!("解析 token JSON 失败: {}", e))?;
    Ok(Some(record))
}

/// 删除厂商 token 文件（撤销认证，不存在视为成功）
pub(super) async fn delete_provider_auth(provider_id: &str) -> Result<(), String> {
    super::super::paths::validate_provider_id(provider_id)?;

    let path = super::super::paths::auth_file_path(provider_id);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("删除 token 文件失败: {}", e))?;
    }
    Ok(())
}

// 通用辅助
/// 当前 Unix 时间戳（秒）
pub(super) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 生成 OAuth2 state 参数（CSRF 防护）
///
/// 基于系统时间纳秒 + 进程 ID 生成，非密码学安全但足以防止本地回调伪造。
pub(super) fn generate_state() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    format!("{:x}{:x}", nanos, pid)
}
