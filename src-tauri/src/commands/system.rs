//! 系统命令

use crate::log_info;
use crate::state::AppState;
use tauri::State;

/// 更新配置并保存
async fn update_config<F>(state: &AppState, updater: F) -> Result<(), String>
where
    F: FnOnce(&mut crate::state::AppConfig),
{
    let mut config = state.config.lock().await;
    updater(&mut config);
    let config_clone = config.clone();
    drop(config);

    // 立即保存到文件
    crate::config::save_config(&config_clone)?;
    Ok(())
}

/// 打开游戏目录
#[tauri::command]
pub async fn open_game_dir(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().await;
    let game_dir = &config.game_dir;

    log_info!("Opening game directory: {}", game_dir);

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
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    Ok(game_dir.to_string_lossy().to_string())
}

/// 选择文件夹（打开系统文件夹选择对话框）
#[tauri::command]
pub async fn select_folder(app: tauri::AppHandle, current: Option<String>) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut dialog = app.dialog().file();
    if let Some(ref dir) = current {
        dialog = dialog.set_directory(dir);
    }

    let result = dialog.blocking_pick_folder();
    Ok(result.map(|p| p.to_string()))
}

/// 选择文件（打开系统文件选择对话框）
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
    log_info!("Game directory changed to: {}", game_dir);
    update_config(&state, |config| {
        config.game_dir = game_dir;
    }).await
}

/// 获取系统内存信息
#[tauri::command]
pub async fn get_system_memory() -> Result<crate::minecraft::system::SystemMemory, String> {
    Ok(crate::minecraft::system::get_system_memory())
}

/// 设置镜像源
#[tauri::command]
pub async fn set_mirror_url(
    state: State<'_, AppState>,
    mirror_url: Option<String>,
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    log_info!("Mirror URL changed to: {:?}", mirror_url);
    update_config(&state, |config| {
        config.mirror_url = mirror_url;
    }).await
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
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    let bmclapi = crate::minecraft::sources::BMCLAPI_BASE;

    log_info!("Download source changed to: {}", source);
    update_config(&state, |config| {
        match source.as_str() {
            "mirror" => {
                config.mirror_url_meta = Some(bmclapi.to_string());
                config.mirror_url_download = Some(bmclapi.to_string());
                config.mirror_url = Some(bmclapi.to_string());
                config.mirror_mode = 0;
            }
            "official" => {
                config.mirror_url_meta = None;
                config.mirror_url_download = None;
                config.mirror_url = None;
                config.mirror_mode = 0;
            }
            "smart" => {
                config.mirror_url_meta = None;
                config.mirror_url_download = None;
                config.mirror_url = None;
                config.mirror_mode = 1;
            }
            _ => {}
        }
        config.download_source = source;
    }).await
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
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    log_info!("Max download speed changed to: {} bytes/sec", speed);
    update_config(&state, |config| {
        config.max_download_speed = speed;
    }).await
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
    let storage = crate::storage::Storage::instance();
    Ok(storage.config_path().to_string_lossy().to_string())
}

/// 手动保存配置到文件
#[tauri::command]
pub async fn save_config_to_file(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().await;
    crate::config::save_config(&config)?;
    log_info!("Config saved manually");
    Ok(())
}

/// 设置最小内存
#[tauri::command]
pub async fn set_min_memory(
    state: State<'_, AppState>,
    memory: u32,
) -> Result<(), String> {
    log_info!("Min memory changed to: {} MB", memory);
    update_config(&state, |config| {
        config.min_memory = memory;
    }).await
}

/// 设置最大内存
#[tauri::command]
pub async fn set_max_memory(
    state: State<'_, AppState>,
    memory: u32,
) -> Result<(), String> {
    log_info!("Max memory changed to: {} MB", memory);
    update_config(&state, |config| {
        config.max_memory = memory;
    }).await
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
    log_info!("Max download threads changed to: {}", threads);
    update_config(&state, |config| {
        config.max_download_threads = threads;
    }).await
}

/// 获取下载线程数
#[tauri::command]
pub async fn get_max_download_threads(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.max_download_threads)
}

/// 设置分片数量
#[tauri::command]
pub async fn set_chunk_count(
    state: State<'_, AppState>,
    count: u32,
) -> Result<(), String> {
    log_info!("Chunk count changed to: {}", count);
    update_config(&state, |config| {
        config.chunk_count = count;
    }).await
}

/// 获取分片数量
#[tauri::command]
pub async fn get_chunk_count(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.chunk_count)
}

/// 设置版本隔离模式
#[tauri::command]
pub async fn set_isolation_mode(
    state: State<'_, AppState>,
    mode: u32,
) -> Result<(), String> {
    log_info!("Isolation mode changed to: {}", mode);
    update_config(&state, |config| {
        config.isolation_mode = mode;
    }).await
}

/// 获取版本隔离模式
#[tauri::command]
pub async fn get_isolation_mode(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.isolation_mode)
}

/// 获取配置值（从 storage 读取）
#[tauri::command]
pub async fn get_config_value(section: String, key: String) -> Result<Option<String>, String> {
    let storage = crate::storage::Storage::instance();
    Ok(storage.get_config(&section, &key))
}

/// 设置配置值（写入 storage）
#[tauri::command]
pub async fn set_config_value(section: String, key: String, value: String) -> Result<(), String> {
    let storage = crate::storage::Storage::instance();
    storage.set_config(&section, &key, &value).map_err(|e| e.to_string())?;

    // 日志级别热重载
    if section == "Log" && key == "level" {
        if let Ok(level) = value.parse::<u32>() {
            let log_level = match level {
                0 => crate::logger::LogLevel::Error,
                1 => crate::logger::LogLevel::Error,
                2 => crate::logger::LogLevel::Warn,
                3 => crate::logger::LogLevel::Info,
                4 => crate::logger::LogLevel::Debug,
                5 => crate::logger::LogLevel::Trace,
                _ => crate::logger::LogLevel::Info,
            };
            crate::logger::set_level(log_level);
            log_info!("Log level changed to: {}", level);
        }
    }

    Ok(())
}

/// 获取日志级别
#[tauri::command]
pub async fn get_log_level(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.log_level)
}

/// 设置日志级别
#[tauri::command]
pub async fn set_log_level(
    state: State<'_, AppState>,
    level: u32,
) -> Result<(), String> {
    log_info!("Log level changed to: {}", level);
    update_config(&state, |config| {
        config.log_level = level;
    }).await
}

/// 获取代理模式
#[tauri::command]
pub async fn get_proxy_mode(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.proxy_mode.clone())
}

/// 设置代理模式
#[tauri::command]
pub async fn set_proxy_mode(
    state: State<'_, AppState>,
    mode: String,
) -> Result<(), String> {
    log_info!("Proxy mode changed to: {}", mode);
    update_config(&state, |config| {
        config.proxy_mode = mode;
    }).await
}

/// 获取代理类型
#[tauri::command]
pub async fn get_proxy_type(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.proxy_type.clone())
}

/// 设置代理类型
#[tauri::command]
pub async fn set_proxy_type(
    state: State<'_, AppState>,
    proxy_type: String,
) -> Result<(), String> {
    log_info!("Proxy type changed to: {}", proxy_type);
    update_config(&state, |config| {
        config.proxy_type = proxy_type;
    }).await
}

/// 获取代理地址
#[tauri::command]
pub async fn get_proxy_url(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.proxy_url.clone())
}

/// 设置代理地址
#[tauri::command]
pub async fn set_proxy_url(
    state: State<'_, AppState>,
    url: String,
) -> Result<(), String> {
    log_info!("Proxy URL changed to: {}", url);
    update_config(&state, |config| {
        config.proxy_url = url;
    }).await
}
