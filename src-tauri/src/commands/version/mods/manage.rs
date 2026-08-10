//! Mod 管理命令（toggle_mod / delete_mod）
//!
//! 注：原 2 个独立 Tauri 命令已聚合为 `version_mods_manager` IPC 入口，
//! 通过请求体的 `action` 字段分发。本模块函数已去掉 `#[tauri::command]` 标注，
//! 由 `manager::dispatch` 反序列化参数后调用。

use crate::state::AppState;
use crate::{log_error, log_info};

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::get_mods_dir;
use crate::utils::path::sanitize_file_name;

/// 启用/禁用 Mod（重命名文件扩展名）
///
/// 返回重命名后的新文件名（前端据此原地更新 mod 字段，避免重新加载列表丢失预加载的 project 等信息）。
pub async fn toggle_mod(
    state: &AppState,
    version_id: String,
    file_name: String,
    enable: bool,
) -> Result<String, String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    log_info!(
        "Toggling mod {} for version {} (enable={})",
        file_name,
        version_id,
        enable
    );

    let mods_dir = get_mods_dir(state, &version_id).await?;
    pack_common::toggle_entry(&mods_dir, &file_name, enable).map_err(|e| {
        log_error!("Failed to toggle mod: {}", e);
        e
    })
}

/// 删除 Mod 文件
pub async fn delete_mod(
    state: &AppState,
    version_id: String,
    file_name: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    log_info!("Deleting mod {} for version {}", file_name, version_id);

    let mods_dir = get_mods_dir(state, &version_id).await?;
    pack_common::delete_entry(&mods_dir, &file_name).map_err(|e| {
        log_error!("Failed to delete mod: {}", e);
        e
    })
}
