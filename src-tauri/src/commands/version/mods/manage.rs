//! Mod 管理命令（toggle_mod / delete_mod）
//!
//! 注：原 2 个独立 Tauri 命令已聚合为 `version_mods_manager` IPC 入口，
//! 通过请求体的 `action` 字段分发。本模块函数已去掉 `#[tauri::command]` 标注，
//! 由 `utils::version_mods_manager::dispatch` 反序列化参数后调用。

use crate::state::AppState;
use crate::{log_error, log_info};

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
    let src_path = mods_dir.join(&file_name);

    if !src_path.exists() {
        return Err(format!("Mod 文件不存在: {}", file_name));
    }

    let lower = file_name.to_lowercase();
    let is_currently_enabled = !(lower.ends_with(".disabled") || lower.ends_with(".old"));

    // 状态已一致，无需操作
    if is_currently_enabled == enable {
        return Ok(file_name);
    }

    // 计算目标文件名
    let new_name = if enable {
        // 启用：去掉 .disabled 或 .old 后缀
        file_name
            .trim_end_matches(".disabled")
            .trim_end_matches(".old")
            .to_string()
    } else {
        // 禁用：优先使用 .disabled，若 .disabled 已存在则用 .old
        let disabled_name = format!("{}.disabled", file_name);
        if !mods_dir.join(&disabled_name).exists() {
            disabled_name
        } else {
            format!("{}.old", file_name)
        }
    };

    let dst_path = mods_dir.join(&new_name);

    // 目标已存在（同名文件冲突）
    if dst_path.exists() && dst_path != src_path {
        return Err(format!("目标文件已存在: {}", new_name));
    }

    std::fs::rename(&src_path, &dst_path).map_err(|e| {
        log_error!("Failed to toggle mod: {}", e);
        e.to_string()
    })?;

    log_info!("Mod renamed: {} -> {}", file_name, new_name);
    Ok(new_name)
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
    let path = mods_dir.join(&file_name);

    if !path.exists() {
        return Err(format!("Mod 文件不存在: {}", file_name));
    }

    std::fs::remove_file(&path).map_err(|e| {
        log_error!("Failed to delete mod: {}", e);
        e.to_string()
    })?;

    log_info!("Mod deleted: {}", file_name);
    Ok(())
}
