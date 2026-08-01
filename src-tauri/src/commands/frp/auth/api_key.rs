//! API Key 认证流程
//!
//! 用户手动获取 Key 填入，直接作为 access_token 存储（SDK DES 加密写入文件），
//! 无过期时间、无 refresh_token。调用厂商 API 时由 api_spec 模块注入请求头。

use super::super::provider::{read_provider_manifest, resolve_auth_type};
use super::storage::store_token_info;
use crate::log_info;

/// 保存 API Key（auth_type=api_key 时由前端调用）
pub(super) async fn save_api_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    let manifest = read_provider_manifest(provider_id)?;
    if resolve_auth_type(provider_id, &manifest) != "api_key" {
        return Err(format!("厂商 {} 不使用 API Key 认证", provider_id));
    }
    if api_key.trim().is_empty() {
        return Err("API Key 不能为空".to_string());
    }

    store_token_info(provider_id, api_key.trim(), None, None, None).await?;
    log_info!("[Frp Auth] API Key 已保存: provider={}", provider_id);
    Ok(())
}
