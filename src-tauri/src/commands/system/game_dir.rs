//! 游戏目录相关命令

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
    open_path_impl(&game_dir.to_string_lossy())
}

/// 打开任意路径（文件夹或文件）
#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    log_info!("Opening path: {}", path);
    open_path_impl(&path)
}

/// 跨平台打开路径的内部实现（路径必须已存在，不自动创建）
pub fn open_path_impl(path: &str) -> Result<(), String> {
    // 安全校验：拒绝路径遍历
    if path.contains("..") {
        return Err("路径不能包含 ..".to_string());
    }
    // 安全校验：拒绝 UNC 路径（防止 SMB 认证泄露）
    if path.starts_with("\\\\") || path.starts_with("//") {
        return Err("不支持 UNC 路径".to_string());
    }

    let p = std::path::Path::new(path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open path: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open path: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open path: {}", e))?;
    }

    Ok(())
}

/// 在资源管理器中打开并选中指定文件
/// Windows: explorer /select,<file>
/// macOS: open -R <file>
/// Linux: 不支持选中，回退到打开父目录
#[tauri::command]
pub async fn reveal_in_explorer(path: String) -> Result<(), String> {
    log_info!("Reveal in explorer: {}", path);
    // 安全校验：拒绝路径遍历
    if path.contains("..") {
        return Err("路径不能包含 ..".to_string());
    }
    // 安全校验：拒绝 UNC 路径
    if path.starts_with("\\\\") || path.starts_with("//") {
        return Err("不支持 UNC 路径".to_string());
    }

    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        // explorer /select,<file> 会在资源管理器中打开父目录并选中文件
        // 路径需要用逗号分隔（不能用 /select <file> 空格形式，因路径可能含空格）
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| format!("Failed to reveal in explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: open -R <file> 在 Finder 中显示文件
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| format!("Failed to reveal in finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 文件管理器没有统一的"选中文件"接口，回退到打开父目录
        let parent = p.parent().unwrap_or(std::path::Path::new("."));
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
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
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut dialog = app.dialog().file();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
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

/// 更新游戏目录
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
