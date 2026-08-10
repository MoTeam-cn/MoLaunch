//! Mod 安装与文件操作命令（install_mod / open_mods_dir / get_version_mods_dir / reveal_mod_file）
//!
//! 注：原 4 个独立 Tauri 命令已聚合为 `version_mods_manager` IPC 入口，
//! 通过请求体的 `action` 字段分发。本模块函数已去掉 `#[tauri::command]` 标注，
//! 由 `manager::dispatch` 反序列化参数后调用。

use crate::error_util::log_err;
use crate::state::AppState;
use crate::{log_error, log_info};

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::get_mods_dir;
use crate::utils::path::sanitize_file_name;

/// 从外部文件安装 Mod（复制到 mods 目录）
pub async fn install_mod(
    state: &AppState,
    version_id: String,
    source_path: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Installing mod to version {}", version_id);

    let mods_dir = get_mods_dir(state, &version_id).await?;
    let clean_name = pack_common::install_entry(&mods_dir, &source_path, &["jar", "litemod"])
        .map_err(|e| {
            log_error!("Failed to install mod: {}", e);
            e
        })?;
    log_info!("Mod installed: {}", clean_name);
    Ok(())
}

/// 打开版本的 mods 目录（自动创建）
pub async fn open_mods_dir(state: &AppState, version_id: String) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    let mods_dir = get_mods_dir(state, &version_id).await?;

    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(log_err("Failed to create mods directory"))?;
    }

    let path_str = mods_dir.to_string_lossy().to_string();
    log_info!("Opening mods dir: {}", path_str);
    crate::minecraft::system::shell::open_path(&path_str)
}

/// 获取版本的 mods 目录路径（不打开，用于前端下载 mod 时指定保存位置）
pub async fn get_version_mods_dir(state: &AppState, version_id: String) -> Result<String, String> {
    sanitize_version_id(&version_id)?;
    let mods_dir = get_mods_dir(state, &version_id).await?;
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(log_err("Failed to create mods directory"))?;
    }
    Ok(mods_dir.to_string_lossy().to_string())
}

/// 在资源管理器中打开并选中指定 Mod 文件
pub async fn reveal_mod_file(
    state: &AppState,
    version_id: String,
    file_name: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    let mods_dir = get_mods_dir(state, &version_id).await?;
    let mod_path = mods_dir.join(&file_name);
    if !mod_path.exists() {
        return Err(format!("Mod 文件不存在: {}", file_name));
    }
    let path_str = mod_path.to_string_lossy().to_string();
    log_info!("Revealing mod file: {}", path_str);
    crate::minecraft::system::shell::reveal_in_file_manager(&path_str)
}
