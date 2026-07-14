//! 配置文件相关命令

use crate::log_info;
use crate::state::AppState;
use tauri::State;

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

/// 获取配置值（从 storage 读取）
#[tauri::command]
pub async fn get_config_value(section: String, key: String) -> Result<Option<String>, String> {
    let storage = crate::storage::Storage::instance();
    Ok(storage.get_config(&section, &key))
}

/// 校验 section/key 是否在白名单中（基于 config.ini 实际配置项）
fn is_valid_config_key(section: &str, key: &str) -> bool {
    match (section, key) {
        // General 段
        ("General", "game_dir") => true,
        ("General", "theme") => true,
        ("General", "language") => true,
        ("General", "isolation_mode") => true,
        // Folders 段
        ("Folders", "list") => true,
        // Java 段
        ("Java", "path") => true,
        // Download 段
        ("Download", "max_threads") => true,
        ("Download", "max_speed") => true,
        ("Download", "source") => true,
        ("Download", "mirror_mode") => true,
        ("Download", "chunk_count") => true,
        // Mirror 段
        ("Mirror", "url") => true,
        ("Mirror", "url_meta") => true,
        ("Mirror", "url_download") => true,
        // Memory 段
        ("Memory", "mode") => true,
        ("Memory", "min") => true,
        ("Memory", "max") => true,
        // Log 段
        ("Log", "level") => true,
        // Proxy 段
        ("Proxy", "mode") => true,
        ("Proxy", "type") => true,
        ("Proxy", "url") => true,
        _ => false,
    }
}

/// 设置配置值（写入 storage）
#[tauri::command]
pub async fn set_config_value(
    state: State<'_, AppState>,
    section: String,
    key: String,
    value: String,
) -> Result<(), String> {
    // 白名单校验：拒绝不在白名单中的 section/key
    if !is_valid_config_key(&section, &key) {
        return Err(format!("不支持的配置项: [{}] {}", section, key));
    }

    let storage = crate::storage::Storage::instance();
    storage
        .set_config(&section, &key, &value)
        .map_err(|e| e.to_string())?;

    // 日志级别热重载：同时更新 logger 运行时级别 + state.config 内存
    // 否则后续 save_config(&state.config) 会用内存中的旧 log_level 覆盖 storage 的新值
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
            // 同步刷新内存中的 AppConfig.log_level，避免 save_config 覆盖
            let mut config = state.config.lock().await;
            config.log_level = level;
            log_info!("Log level changed to: {}", level);
        }
    }

    Ok(())
}
