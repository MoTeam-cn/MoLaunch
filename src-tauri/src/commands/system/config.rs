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
