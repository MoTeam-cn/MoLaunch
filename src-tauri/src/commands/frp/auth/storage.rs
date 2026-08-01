//! keyring 密钥存储辅助 + token 上下文工具
//!
//! service=`frp:<provider_id>`，username=`access_token` / `refresh_token` /
//! `expires_at` / `scopes`。token 过期前 5 分钟自动刷新由调用方负责。

use super::super::{ApiKeyConfig, AuthConfig, DeviceCodeConfig, OAuth2Config};
use crate::log_error;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// keyring 密钥存储辅助
// ============================================================

/// 密钥存储的 key 列表
pub(super) const KEY_ACCESS_TOKEN: &str = "access_token";
pub(super) const KEY_REFRESH_TOKEN: &str = "refresh_token";
pub(super) const KEY_EXPIRES_AT: &str = "expires_at";
pub(super) const KEY_SCOPES: &str = "scopes";

/// 构造 keyring Entry
///
/// service = `frp:<provider_id>`，username = 具体键名。
/// keyring 不可用时返回明确错误。
fn keyring_entry(provider_id: &str, key: &str) -> Result<keyring::Entry, String> {
    let service = format!("frp:{}", provider_id);
    keyring::Entry::new(&service, key).map_err(|e| {
        log_error!(
            "[Frp Auth] keyring 不可用 (provider={}): {}",
            provider_id,
            e
        );
        format!("OS 密钥存储不可用: {}", e)
    })
}

/// 存储 token 值到 keyring
pub(super) fn store_secret(provider_id: &str, key: &str, value: &str) -> Result<(), String> {
    let entry = keyring_entry(provider_id, key)?;
    entry.set_password(value).map_err(|e| {
        log_error!(
            "[Frp Auth] 存储密钥失败 (provider={}, key={}): {}",
            provider_id,
            key,
            e
        );
        format!("存储密钥失败: {}", e)
    })
}

/// 读取 token 值（不存在返回 None，keyring 不可用返回 Err）
pub(super) fn load_secret(provider_id: &str, key: &str) -> Result<Option<String>, String> {
    let entry = keyring_entry(provider_id, key)?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => {
            log_error!(
                "[Frp Auth] 读取密钥失败 (provider={}, key={}): {}",
                provider_id,
                key,
                e
            );
            Err(format!("OS 密钥存储不可用: {}", e))
        }
    }
}

/// 删除 token 值（不存在视为成功）
pub(super) fn delete_secret(provider_id: &str, key: &str) -> Result<(), String> {
    let entry = keyring_entry(provider_id, key)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("删除密钥失败: {}", e)),
    }
}

// ============================================================
// token 存储辅助（封装 access_token / refresh_token / expires_at / scopes）
// ============================================================

/// 存储完整 token 信息
pub(super) fn store_token_info(
    provider_id: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in: Option<u64>,
    scopes: Option<&Vec<String>>,
) -> Result<(), String> {
    store_secret(provider_id, KEY_ACCESS_TOKEN, access_token)?;
    if let Some(rt) = refresh_token {
        store_secret(provider_id, KEY_REFRESH_TOKEN, rt)?;
    }
    if let Some(secs) = expires_in {
        let expires_at = now_secs() + secs;
        store_secret(provider_id, KEY_EXPIRES_AT, &expires_at.to_string())?;
    }
    if let Some(sc) = scopes {
        let json = serde_json::to_string(sc).map_err(|e| format!("序列化 scopes 失败: {}", e))?;
        store_secret(provider_id, KEY_SCOPES, &json)?;
    }
    Ok(())
}

/// 读取 token 过期时间（Unix 秒）
pub(super) fn load_expires_at(provider_id: &str) -> Result<Option<u64>, String> {
    match load_secret(provider_id, KEY_EXPIRES_AT)? {
        Some(s) => s
            .parse::<u64>()
            .map(Some)
            .map_err(|e| format!("解析过期时间失败: {}", e)),
        None => Ok(None),
    }
}

/// 读取权限范围
pub(super) fn load_scopes(provider_id: &str) -> Result<Option<Vec<String>>, String> {
    match load_secret(provider_id, KEY_SCOPES)? {
        Some(s) => {
            let scopes: Vec<String> =
                serde_json::from_str(&s).map_err(|e| format!("解析 scopes 失败: {}", e))?;
            Ok(Some(scopes))
        }
        None => Ok(None),
    }
}

// ============================================================
// 配置取值辅助
// ============================================================

/// 获取 OAuth2Config（不存在则报错）
pub(super) fn require_oauth2_config<'a>(
    auth: &'a AuthConfig,
    provider_id: &str,
) -> Result<&'a OAuth2Config, String> {
    auth.oauth2
        .as_ref()
        .ok_or_else(|| format!("厂商 {} 的 manifest 缺少 auth.oauth2 配置", provider_id))
}

/// 获取 DeviceCodeConfig（不存在则报错）
pub(super) fn require_device_code_config<'a>(
    auth: &'a AuthConfig,
    provider_id: &str,
) -> Result<&'a DeviceCodeConfig, String> {
    auth.device_code.as_ref().ok_or_else(|| {
        format!(
            "厂商 {} 的 manifest 缺少 auth.device_code 配置",
            provider_id
        )
    })
}

/// 获取 ApiKeyConfig（不存在则报错）
#[allow(dead_code)]
pub(super) fn require_api_key_config<'a>(
    auth: &'a AuthConfig,
    provider_id: &str,
) -> Result<&'a ApiKeyConfig, String> {
    auth.api_key
        .as_ref()
        .ok_or_else(|| format!("厂商 {} 的 manifest 缺少 auth.api_key 配置", provider_id))
}

// ============================================================
// 通用辅助
// ============================================================

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

/// 解析 scope 字符串（空格分隔）为 Vec
pub(super) fn parse_scopes(scope_str: &str) -> Vec<String> {
    scope_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}
