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
        // Plugin 段已迁移至 AppData 常驻化存储（personalization.json），不再使用 INI
        _ => false,
    }
}

/// 设置配置值（写入 storage + 同步内存 AppConfig）
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

    // 同步刷新内存中的 AppConfig，避免后续 save_config 用内存旧值覆盖 INI 新值
    // （此前仅 Log/level 做了特例补丁，其余字段存在数据覆盖风险）
    let mut config = state.config.lock().await;
    match (section.as_str(), key.as_str()) {
        ("General", "game_dir") => config.game_dir = value.clone(),
        ("General", "theme") => config.theme = value.clone(),
        ("General", "language") => config.language = value.clone(),
        ("General", "isolation_mode") => {
            config.isolation_mode = value.parse().unwrap_or(0);
        }
        ("Java", "path") => {} // Java path 不在 AppConfig 中，走 INI [Java] 独立存储
        ("Download", "max_threads") => {
            config.max_download_threads = value.parse().unwrap_or(0);
        }
        ("Download", "max_speed") => {
            config.max_download_speed = value.parse().unwrap_or(0);
        }
        ("Download", "source") => config.download_source = value.clone(),
        ("Download", "meta_source") => config.meta_source = value.clone(),
        ("Download", "mirror_mode") => {
            config.mirror_mode = value.parse().unwrap_or(0);
        }
        ("Download", "chunk_count") => {
            config.chunk_count = value.parse().unwrap_or(0);
        }
        ("Mirror", "url") => {
            config.mirror_url = if value.is_empty() {
                None
            } else {
                Some(value.clone())
            }
        }
        ("Mirror", "url_meta") => {
            config.mirror_url_meta = if value.is_empty() {
                None
            } else {
                Some(value.clone())
            }
        }
        ("Mirror", "url_download") => {
            config.mirror_url_download = if value.is_empty() {
                None
            } else {
                Some(value.clone())
            }
        }
        ("Memory", "mode") => config.memory_mode = value.clone(),
        ("Memory", "min") => {
            config.min_memory = value.parse().unwrap_or(0);
        }
        ("Memory", "max") => {
            config.max_memory = value.parse().unwrap_or(0);
        }
        ("Log", "level") => {
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
                config.log_level = level;
                log_info!("Log level changed to: {}", level);
            }
        }
        ("Proxy", "mode") => config.proxy_mode = value.clone(),
        ("Proxy", "type") => config.proxy_type = value.clone(),
        ("Proxy", "url") => config.proxy_url = value.clone(),
        _ => {}
    }

    Ok(())
}
