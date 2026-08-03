//! Mod 安装与文件操作命令（install_mod / open_mods_dir / get_version_mods_dir / reveal_mod_file）
//!
//! 注：原 4 个独立 Tauri 命令已聚合为 `version_mods_manager` IPC 入口，
//! 通过请求体的 `action` 字段分发。本模块函数已去掉 `#[tauri::command]` 标注，
//! 由 `manager::dispatch` 反序列化参数后调用。

use crate::error_util::log_err;
use crate::state::AppState;
use crate::{log_error, log_info};

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

    // 安全校验：源路径不能包含 ..
    if !crate::utils::path::is_safe_relative_path(&source_path) {
        return Err("源路径不能包含 ..".to_string());
    }

    log_info!("Installing mod to version {}", version_id);

    let mods_dir = get_mods_dir(state, &version_id).await?;

    // 确保 mods 目录存在
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(|e| {
            log_error!("Failed to create mods dir: {}", e);
            e.to_string()
        })?;
    }

    let src = std::path::Path::new(&source_path);
    if !src.is_absolute() {
        return Err("源路径必须是绝对路径".to_string());
    }
    if !src.exists() {
        return Err(format!("源文件不存在: {}", source_path));
    }

    // 提取文件名，去除 .disabled / .old 后缀
    let original_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无法获取文件名")?
        .to_string();
    let clean_name = original_name
        .trim_end_matches(".disabled")
        .trim_end_matches(".old")
        .to_string();

    // 校验为 Mod 文件
    let lower = clean_name.to_lowercase();
    if !(lower.ends_with(".jar") || lower.ends_with(".litemod")) {
        return Err("仅支持 .jar 或 .litemod 格式的 Mod 文件".to_string());
    }

    let dst = mods_dir.join(&clean_name);

    // 若目标已存在，跳过（避免覆盖）
    if dst.exists() {
        return Err(format!("Mods 目录已存在同名文件: {}", clean_name));
    }

    log_info!("Installing mod from {} to {}", source_path, dst.display());

    std::fs::copy(src, &dst).map_err(|e| {
        log_error!("Failed to copy mod: {}", e);
        e.to_string()
    })?;

    log_info!("Mod installed: {} -> {}", source_path, clean_name);
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
