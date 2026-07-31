//! API Key 认证流程
//!
//! 用户手动获取 Key 填入，直接作为 access_token 存储到 OS 密钥存储，
//! 无过期时间、无 refresh_token。调用厂商 API 时由 api_schema 模块注入请求头。

use super::super::provider::read_provider_manifest;
use super::storage::{store_secret, KEY_ACCESS_TOKEN};
use crate::log_info;

/// 保存 API Key（auth_type=api_key 时由前端调用）
pub(super) async fn save_api_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    let manifest = read_provider_manifest(provider_id)?;
    if manifest.auth.auth_type != "api_key" {
        return Err(format!("厂商 {} 不使用 API Key 认证", provider_id));
    }
    if api_key.trim().is_empty() {
        return Err("API Key 不能为空".to_string());
    }

    store_secret(provider_id, KEY_ACCESS_TOKEN, api_key.trim())?;
    log_info!("[Frp Auth] API Key 已保存: provider={}", provider_id);
    Ok(())
}
