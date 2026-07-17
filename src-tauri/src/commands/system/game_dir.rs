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

    // 如果游戏目录不存在，先尝试创建（防御性：启动时创建可能因权限失败，这里兜底）
    if !game_dir.exists() {
        std::fs::create_dir_all(&game_dir).map_err(|e| format!("创建游戏目录失败: {}", e))?;
    }

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

    log_info!("Opening path: {}", path);

    #[cfg(target_os = "windows")]
    {
        // 不能用 `explorer <path>`：Rust 会给含空格的路径自动加引号，
        // explorer.exe 对带引号的裸路径解析失败会回退到打开"文档"库。
        // 改用 `cmd /c start "" "<path>"`：start 命令正确处理带引号路径。
        // 第一个 "" 是 start 的窗口标题占位（start 语法要求），CREATE_NO_WINDOW 隐藏 cmd 黑框。
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("cmd")
            .args(["/c", "start", "", path])
            .creation_flags(CREATE_NO_WINDOW)
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
pub fn reveal_in_explorer_impl(path: &str) -> Result<(), String> {
    // 安全校验：拒绝路径遍历
    if path.contains("..") {
        return Err("路径不能包含 ..".to_string());
    }
    // 安全校验：拒绝 UNC 路径
    if path.starts_with("\\\\") || path.starts_with("//") {
        return Err("不支持 UNC 路径".to_string());
    }

    let p = std::path::Path::new(path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    log_info!("Reveal in explorer: {}", path);

    #[cfg(target_os = "windows")]
    {
        // explorer /select,<file> 会在资源管理器中打开父目录并选中文件
        // /select, 形式带引号时 explorer 能正确解析（与裸路径不同）：
        //   explorer "/select,C:\path with spaces\file.jar"  ← 可行
        //   explorer "C:\path with spaces\mods"              ← 不可行（回退到文档库）
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| format!("Failed to reveal in explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: open -R <file> 在 Finder 中显示文件
        std::process::Command::new("open")
            .args(["-R", path])
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

/// Tauri 命令包装：在资源管理器中打开并选中指定文件
#[tauri::command]
pub async fn reveal_in_explorer(path: String) -> Result<(), String> {
    log_info!("Reveal in explorer: {}", path);
    reveal_in_explorer_impl(&path)
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
