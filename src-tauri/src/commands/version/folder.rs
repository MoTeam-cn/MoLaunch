//! Minecraft 文件夹管理命令
//!
//! 支持多文件夹管理：
//! - list_mc_folders：列出所有文件夹
//! - add_mc_folder：添加文件夹（自动去重）
//! - remove_mc_folder：移除文件夹
//! - switch_mc_folder：切换当前文件夹
//! - rename_mc_folder：重命名文件夹

use crate::config::save_config;
use crate::state::{AppState, McFolder};
use crate::{log_info, log_warn};
use tauri::State;

/// 列出所有 Minecraft 文件夹
///
/// 返回的 path 统一为绝对路径，便于前端比较和展示
#[tauri::command]
pub async fn list_mc_folders(state: State<'_, AppState>) -> Result<Vec<McFolder>, String> {
    let config = state.config.lock().await;
    let folders: Vec<McFolder> = config
        .mc_folders
        .iter()
        .map(|f| McFolder {
            name: f.name.clone(),
            path: crate::state::resolve_game_dir(&f.path)
                .to_string_lossy()
                .to_string(),
        })
        .collect();
    Ok(folders)
}

/// 添加 Minecraft 文件夹
///
/// - 自动去重：路径相同则更新名称
/// - 路径规范化：统一为绝对路径存储
#[tauri::command]
pub async fn add_mc_folder(
    state: State<'_, AppState>,
    name: String,
    path: String,
) -> Result<Vec<McFolder>, String> {
    if name.trim().is_empty() {
        return Err("文件夹名称不能为空".to_string());
    }
    if path.trim().is_empty() {
        return Err("文件夹路径不能为空".to_string());
    }

    let name = name.trim().to_string();
    // 路径规范化：转换为绝对路径
    let path = crate::state::resolve_game_dir(path.trim())
        .to_string_lossy()
        .to_string();

    log_info!("Adding MC folder: {} -> {}", name, path);

    let mut config = state.config.lock().await;

    // 去重：路径相同则更新名称
    let mut found = false;
    for folder in &mut config.mc_folders {
        let existing = crate::state::resolve_game_dir(&folder.path);
        if existing == std::path::Path::new(&path) {
            folder.name = name.clone();
            found = true;
            log_info!("Folder path already exists, updated name");
            break;
        }
    }

    if !found {
        config.mc_folders.push(McFolder {
            name,
            path: path.clone(),
        });
    }

    let result = config.mc_folders.clone();

    // 持久化
    if let Err(e) = save_config(&config) {
        log_warn!("Failed to save config after adding folder: {}", e);
    }

    Ok(result)
}

/// 移除 Minecraft 文件夹
///
/// - 不允许移除最后一个文件夹
/// - 如果移除的是当前文件夹，自动切换到第一个
#[tauri::command]
pub async fn remove_mc_folder(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<McFolder>, String> {
    log_info!("Removing MC folder: {}", path);

    let mut config = state.config.lock().await;

    if config.mc_folders.len() <= 1 {
        return Err("至少需要保留一个文件夹".to_string());
    }

    // 查找并移除
    let target_path = crate::state::resolve_game_dir(&path);
    let initial_len = config.mc_folders.len();
    config.mc_folders.retain(|f| {
        crate::state::resolve_game_dir(&f.path) != target_path
    });

    if config.mc_folders.len() == initial_len {
        return Err("未找到要移除的文件夹".to_string());
    }

    // 如果移除的是当前文件夹，切换到第一个
    let current_resolved = crate::state::resolve_game_dir(&config.game_dir);
    if current_resolved == target_path {
        if let Some(first) = config.mc_folders.first() {
            config.game_dir = first.path.clone();
            log_info!("Current folder removed, switched to: {}", config.game_dir);
        }
    }

    let result = config.mc_folders.clone();

    if let Err(e) = save_config(&config) {
        log_warn!("Failed to save config after removing folder: {}", e);
    }

    Ok(result)
}

/// 切换当前 Minecraft 文件夹
#[tauri::command]
pub async fn switch_mc_folder(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    log_info!("Switching MC folder to: {}", path);

    let mut config = state.config.lock().await;

    // 验证路径在列表中存在
    let target_path = crate::state::resolve_game_dir(&path);
    let exists = config
        .mc_folders
        .iter()
        .any(|f| crate::state::resolve_game_dir(&f.path) == target_path);

    if !exists {
        return Err("目标文件夹不在列表中".to_string());
    }

    config.game_dir = path.clone();

    if let Err(e) = save_config(&config) {
        log_warn!("Failed to save config after switching folder: {}", e);
    }

    Ok(path)
}

/// 重命名 Minecraft 文件夹
#[tauri::command]
pub async fn rename_mc_folder(
    state: State<'_, AppState>,
    path: String,
    new_name: String,
) -> Result<Vec<McFolder>, String> {
    if new_name.trim().is_empty() {
        return Err("名称不能为空".to_string());
    }

    log_info!("Renaming MC folder {} -> {}", path, new_name);

    let mut config = state.config.lock().await;
    let target_path = crate::state::resolve_game_dir(&path);

    let mut found = false;
    for folder in &mut config.mc_folders {
        if crate::state::resolve_game_dir(&folder.path) == target_path {
            folder.name = new_name.trim().to_string();
            found = true;
            break;
        }
    }

    if !found {
        return Err("未找到要重命名的文件夹".to_string());
    }

    let result = config.mc_folders.clone();

    if let Err(e) = save_config(&config) {
        log_warn!("Failed to save config after renaming folder: {}", e);
    }

    Ok(result)
}
