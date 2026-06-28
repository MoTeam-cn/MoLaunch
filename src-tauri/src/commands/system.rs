//! 系统命令

use crate::state::AppState;
use tauri::State;

/// 重新初始化 SDK 的辅助函数
async fn reinitialize_sdk(state: &AppState) -> Result<String, String> {
    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    let max_threads = config.max_download_threads;
    let log_level = config.log_level;
    let mirror_url = config.mirror_url.clone();
    let mirror_url_meta = config.mirror_url_meta.clone();
    let mirror_url_download = config.mirror_url_download.clone();
    let mirror_mode = config.mirror_mode;
    let max_download_speed = config.max_download_speed;
    drop(config);

    let mut sdk_guard = state.sdk.lock().await;
    if sdk_guard.is_some() {
        log::info!("Releasing old SDK instance...");
        *sdk_guard = None;
    }

    log::info!("Reinitializing SDK...");
    let mut sdk = crate::sdk::SdkInstance::load().map_err(|e| {
        log::error!("Failed to load SDK: {}", e);
        e.to_string()
    })?;

    sdk.init(
        &game_dir,
        max_threads,
        log_level,
        mirror_url.as_deref(),
        mirror_url_meta.as_deref(),
        mirror_url_download.as_deref(),
        mirror_mode,
        max_download_speed,
    ).map_err(|e| {
        log::error!("Failed to initialize SDK: {}", e);
        e.to_string()
    })?;

    let version = sdk.version().map_err(|e| e.to_string())?;
    log::info!("SDK reinitialized successfully, version: {}", version);

    *sdk_guard = Some(sdk);

    Ok(version)
}

/// 打开游戏目录
#[tauri::command]
pub async fn open_game_dir(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().await;
    let game_dir = &config.game_dir;

    log::info!("Opening game directory: {}", game_dir);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(game_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(game_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(game_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}

/// 获取游戏目录
#[tauri::command]
pub async fn get_game_dir(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.game_dir.clone())
}

/// 选择文件夹（打开系统文件夹选择对话框）
#[tauri::command]
pub async fn select_folder(current: Option<String>) -> Result<Option<String>, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;

    let mut dialog = FileDialogBuilder::new();
    if let Some(ref dir) = current {
        dialog = dialog.set_directory(dir);
    }

    let result = dialog.pick_folder();
    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

/// 选择文件（打开系统文件选择对话框）
#[tauri::command]
pub async fn select_file(
    title: Option<String>,
    filters: Option<Vec<FileFilter>>,
) -> Result<Option<String>, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;

    let mut dialog = FileDialogBuilder::new();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }
    if let Some(f) = filters {
        for filter in f {
            let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &exts);
        }
    }

    let result = dialog.pick_file();
    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

#[derive(serde::Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// 更新游戏目录
#[tauri::command]
pub async fn set_game_dir(
    state: State<'_, AppState>,
    game_dir: String,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    log::info!("Game directory changed: {} -> {}", config.game_dir, game_dir);
    config.game_dir = game_dir;
    Ok(())
}

/// 获取系统内存信息
#[tauri::command]
pub async fn get_system_memory() -> Result<crate::sdk::SystemMemory, String> {
    crate::sdk::get_system_memory_static().map_err(|e| {
        log::error!("Failed to get system memory: {}", e);
        e.to_string()
    })
}

/// 设置镜像源
#[tauri::command]
pub async fn set_mirror_url(
    state: State<'_, AppState>,
    mirror_url: Option<String>,
    skip_reinit: Option<bool>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    log::info!("Mirror URL changed: {:?} -> {:?}", config.mirror_url, mirror_url);
    config.mirror_url = mirror_url;
    drop(config);

    if skip_reinit != Some(true) {
        reinitialize_sdk(&state).await?;
    }

    Ok(())
}

/// 获取镜像源
#[tauri::command]
pub async fn get_mirror_url(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.lock().await;
    Ok(config.mirror_url.clone())
}

/// 设置下载源模式
#[tauri::command]
pub async fn set_download_source(
    state: State<'_, AppState>,
    source: String,
    skip_reinit: Option<bool>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    const BMCLAPI: &str = "https://bmclapi2.bangbang93.com";
    
    match source.as_str() {
        "mirror" => {
            config.mirror_url_meta = Some(BMCLAPI.to_string());
            config.mirror_url_download = Some(BMCLAPI.to_string());
            config.mirror_url = Some(BMCLAPI.to_string());
            config.mirror_mode = 0;
        }
        "official" => {
            config.mirror_url_meta = None;
            config.mirror_url_download = None;
            config.mirror_url = None;
            config.mirror_mode = 0;
        }
        "smart" => {
            // 自动探测：SDK 自动检测官方源速度，慢则降级到 BMCLAPI
            config.mirror_url_meta = None;
            config.mirror_url_download = None;
            config.mirror_url = None;
            config.mirror_mode = 1;
        }
        _ => return Err(format!("Invalid source: {}", source)),
    }
    
    config.download_source = source;
    log::info!("Download source changed to: {}", config.download_source);
    drop(config);

    if skip_reinit != Some(true) {
        reinitialize_sdk(&state).await?;
    }

    Ok(())
}

/// 获取下载源模式
#[tauri::command]
pub async fn get_download_source(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.download_source.clone())
}

/// 设置最大下载速度
#[tauri::command]
pub async fn set_max_download_speed(
    state: State<'_, AppState>,
    speed: u64,
    skip_reinit: Option<bool>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.max_download_speed = speed;
    log::info!("Max download speed changed to: {} bytes/sec", speed);
    drop(config);

    if skip_reinit != Some(true) {
        reinitialize_sdk(&state).await?;
    }

    Ok(())
}

/// 获取最大下载速度
#[tauri::command]
pub async fn get_max_download_speed(state: State<'_, AppState>) -> Result<u64, String> {
    let config = state.config.lock().await;
    Ok(config.max_download_speed)
}

/// 获取配置文件路径
#[tauri::command]
pub async fn get_config_path() -> Result<String, String> {
    Ok(crate::config::get_config_file_path().to_string_lossy().to_string())
}

/// 手动保存配置到文件
#[tauri::command]
pub async fn save_config_to_file(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().await;
    crate::config::save_config(&config)?;
    log::info!("Config saved manually");
    Ok(())
}

/// 手动触发 SDK 重新初始化（用于批量设置更新后）
#[tauri::command]
pub async fn reinitialize_sdk_command(state: State<'_, AppState>) -> Result<String, String> {
    reinitialize_sdk(&state).await
}

/// 设置最小内存
#[tauri::command]
pub async fn set_min_memory(
    state: State<'_, AppState>,
    memory: u32,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    log::info!("Min memory changed: {} -> {}", config.min_memory, memory);
    config.min_memory = memory;
    Ok(())
}

/// 设置最大内存
#[tauri::command]
pub async fn set_max_memory(
    state: State<'_, AppState>,
    memory: u32,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    log::info!("Max memory changed: {} -> {}", config.max_memory, memory);
    config.max_memory = memory;
    Ok(())
}

/// 获取内存配置
#[tauri::command]
pub async fn get_memory_config(state: State<'_, AppState>) -> Result<(u32, u32), String> {
    let config = state.config.lock().await;
    Ok((config.min_memory, config.max_memory))
}

/// 设置下载线程数
#[tauri::command]
pub async fn set_max_download_threads(
    state: State<'_, AppState>,
    threads: u32,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    log::info!("Max download threads changed: {} -> {}", config.max_download_threads, threads);
    config.max_download_threads = threads;
    Ok(())
}

/// 获取下载线程数
#[tauri::command]
pub async fn get_max_download_threads(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.max_download_threads)
}
