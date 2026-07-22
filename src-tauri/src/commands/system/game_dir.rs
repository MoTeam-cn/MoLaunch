//! 游戏目录相关命令
//!
//! Shell 命令（打开文件夹、选中文件等）已统一封装到
//! `crate::minecraft::system::shell`，本模块仅保留 Tauri 命令包装层。

use crate::log_info;
use crate::state::AppState;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// 打开游戏目录
#[tauri::command]
pub async fn open_game_dir(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);

    log_info!("Opening game directory: {}", game_dir.display());

    // 如果游戏目录不存在，先尝试创建（防御性：启动时创建可能因权限失败，这里兜底）
    if !game_dir.exists() {
        std::fs::create_dir_all(&game_dir).map_err(|e| format!("创建游戏目录失败: {}", e))?;
    }

    crate::minecraft::system::shell::open_path(&game_dir.to_string_lossy())
}

/// 打开任意路径（文件夹或文件）
#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    crate::minecraft::system::shell::open_path(&path)
}

/// Tauri 命令包装：在资源管理器中打开并选中指定文件
#[tauri::command]
pub async fn reveal_in_explorer(path: String) -> Result<(), String> {
    crate::minecraft::system::shell::reveal_in_file_manager(&path)
}

/// 获取游戏目录
#[tauri::command]
pub async fn get_game_dir(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    Ok(game_dir.to_string_lossy().to_string())
}

/// 选择文件夹
#[tauri::command]
pub async fn select_folder(
    app: tauri::AppHandle,
    current: Option<String>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut dialog = app.dialog().file();
    if let Some(ref dir) = current {
        dialog = dialog.set_directory(dir);
    }

    let result = dialog.blocking_pick_folder();
    Ok(result.map(|p| p.to_string()))
}

/// 选择文件
#[tauri::command]
pub async fn select_file(
    app: tauri::AppHandle,
    title: Option<String>,
    filters: Option<Vec<FileFilter>>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut dialog = app.dialog().file();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }
    if let Some(f) = filters {
        for filter in f {
            let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &exts);
        }
    }

    let result = dialog.blocking_pick_file();
    Ok(result.map(|p| p.to_string()))
}

/// 保存文件对话框（让用户选择保存位置）
#[tauri::command]
pub async fn save_file(
    app: tauri::AppHandle,
    title: Option<String>,
    default_name: Option<String>,
    filters: Option<Vec<FileFilter>>,
    default_directory: Option<String>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut dialog = app.dialog().file();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }
    if let Some(d) = default_directory {
        // 设置对话框默认打开的目录（例如从 ModTab 打开时默认到整合包的 mods 文件夹）
        let path = std::path::PathBuf::from(&d);
        if path.exists() {
            dialog = dialog.set_directory(&path);
        }
    }
    if let Some(n) = default_name {
        dialog = dialog.set_file_name(&n);
    }
    if let Some(f) = filters {
        for filter in f {
            let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &exts);
        }
    }

    let result = dialog.blocking_save_file();
    Ok(result.map(|p| p.to_string()))
}

/// 写入文本文件到指定路径
///
/// 用于导出示例文件（插件模板、布局示例等），路径通常由 `save_file` 对话框返回。
/// 若文件已存在则覆盖，父目录不存在时自动创建。
#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(&path);

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        crate::utils::fs::ensure_dir(parent)?;
    }

    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    log_info!("[System] Text file written: {}", path.display());
    Ok(())
}

/// 更新游戏目录（保留独立命令，因为调用方 version/manage.rs 在切换版本时直接调用）
///
/// 重构说明：此命令保留，未合并到 `apply_config`，因为它在版本切换流程中被
/// 内部逻辑调用，不是用户设置页触发的。用户在设置页改 game_dir 时走
/// `apply_config({ gameDir: ... })`。
#[tauri::command]
pub async fn set_game_dir(state: State<'_, AppState>, game_dir: String) -> Result<(), String> {
    log_info!("Game directory changed to: {}", game_dir);
    super::update_config(&state, |config| {
        config.game_dir = game_dir;
    })
    .await
}

/// 获取系统内存信息
#[tauri::command]
pub async fn get_system_memory() -> Result<crate::minecraft::system::SystemMemory, String> {
    Ok(crate::minecraft::system::get_system_memory())
}
