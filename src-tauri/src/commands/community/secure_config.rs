//! CurseForge API Key 加密配置命令
//!
//! 提供给前端的 CurseForge API Key 读写命令：
//! - `get_curseforge_config`: 读取缓存的 (enabled, api_key)
//! - `set_curseforge_config`: 加密写入 INI + 更新缓存

use crate::minecraft::community::secure_storage;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfConfig {
    pub enabled: bool,
    pub api_key: String,
}

/// 读取 CurseForge 配置（从内存缓存读，已解密）
#[tauri::command]
pub async fn get_curseforge_config() -> Result<CfConfig, String> {
    let (enabled, api_key) = secure_storage::get_cached();
    Ok(CfConfig {
        enabled,
        api_key: api_key.unwrap_or_default(),
    })
}

/// 保存 CurseForge 配置（API Key 加密后写入 INI + 更新缓存）
///
/// SDK 从 AppState 获取（与 AuthStorage 一致，不持有全局引用）
#[tauri::command]
pub async fn set_curseforge_config(
    state: State<'_, AppState>,
    enabled: bool,
    api_key: String,
) -> Result<(), String> {
    secure_storage::save(state.sdk.clone(), enabled, &api_key).await
}
