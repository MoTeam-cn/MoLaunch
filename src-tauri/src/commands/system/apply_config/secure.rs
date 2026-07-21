//! 加密 / 注册表字段分流
//!
//! CurseForge API Key 走 secure_storage（SDK DES 加密），不进 AppConfig；
//! 开发者模式走注册表（DeveloperUnlocked / DeveloperMode），不进 AppConfig。
//! 这两块在 `apply_config_inner` 中先于普通字段更新执行。

use super::super::developer::{KEY_DEV_MODE, KEY_DEV_UNLOCKED};
use super::types::ConfigPatch;
use crate::log_info;
use crate::state::AppState;

/// 读取 CurseForge 配置（异步触发 SDK DES 解密并缓存）
pub async fn read_curseforge() -> (bool, Option<String>) {
    crate::minecraft::community::secure_storage::get_config_async().await
}

/// 读取开发者模式状态（注册表）：(是否已解锁, 是否已开启)
pub fn read_developer() -> (bool, bool) {
    let unlocked = crate::storage::registry::reg_get_bool(KEY_DEV_UNLOCKED);
    let mode = unlocked && crate::storage::registry::reg_get_bool(KEY_DEV_MODE);
    (unlocked, mode)
}

/// 应用 CurseForge 配置（加密存储，不进 AppConfig）
///
/// 至少一个 CF 字段要更新时进入：取 patch 提供的值，未提供的字段异步读取旧值，
/// 使用 `get_config_async` 确保首次保存时 api_key 已解密（避免误清空）。
pub async fn apply_curseforge(state: &AppState, patch: &ConfigPatch) -> Result<(), String> {
    if patch.curseforge_enabled.is_none() && patch.curseforge_api_key.is_none() {
        return Ok(());
    }
    let (old_enabled, old_key) = read_curseforge().await;
    let enabled = patch.curseforge_enabled.unwrap_or(old_enabled);
    let api_key = match &patch.curseforge_api_key {
        Some(k) => k.clone(),
        None => old_key.unwrap_or_default(),
    };
    log_info!("[Config] CurseForge 配置更新: enabled={}", enabled);
    crate::minecraft::community::secure_storage::save(state.sdk.clone(), enabled, &api_key).await
}

/// 应用开发者模式（注册表，不进 AppConfig）
///
/// 仅在已解锁时可生效（与原 `set_developer_mode` 命令行为一致）。
pub fn apply_developer_mode(patch: &ConfigPatch) -> Result<(), String> {
    if let Some(enabled) = patch.developer_mode {
        let unlocked = crate::storage::registry::reg_get_bool(KEY_DEV_UNLOCKED);
        if !unlocked {
            return Err("开发者模式尚未解锁".to_string());
        }
        log_info!("[Config] developer_mode = {}", enabled);
        crate::storage::registry::reg_set_bool(KEY_DEV_MODE, enabled)
            .map_err(|e| format!("写入注册表失败: {}", e))?;
    }
    Ok(())
}
