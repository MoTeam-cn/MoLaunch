//! 配置持久化模块
//!
//! 使用 storage 模块管理配置文件（INI 格式）

use crate::{log_info, log_warn, log_debug};
use crate::state::AppConfig;
use crate::storage::Storage;

/// 从 storage 加载配置
pub fn load_config() -> Result<Option<AppConfig>, String> {
    let storage = Storage::instance();

    // 初始化 storage
    if let Err(e) = storage.init() {
        return Err(format!("Failed to initialize storage: {}", e));
    }

    // 读取配置
    let config = match storage.read_config() {
        Ok(config) => config,
        Err(e) => {
            log_warn!("Failed to read config: {}", e);
            return Ok(None);
        }
    };

    let mut app_config = AppConfig::default();

    // General
    app_config.game_dir = config.get_or("General", "game_dir", &app_config.game_dir);
    app_config.theme = config.get_or("General", "theme", &app_config.theme);
    app_config.language = config.get_or("General", "language", &app_config.language);

    // Download
    if let Some(threads) = config.get("Download", "max_threads") {
        app_config.max_download_threads = threads.parse().unwrap_or(app_config.max_download_threads);
    }
    if let Some(speed) = config.get("Download", "max_speed") {
        app_config.max_download_speed = speed.parse().unwrap_or(app_config.max_download_speed);
    }
    app_config.download_source = config.get_or("Download", "source", &app_config.download_source);
    if let Some(mode) = config.get("Download", "mirror_mode") {
        app_config.mirror_mode = mode.parse().unwrap_or(app_config.mirror_mode);
    }

    // Mirror
    app_config.mirror_url = config.get("Mirror", "url");
    app_config.mirror_url_meta = config.get("Mirror", "url_meta");
    app_config.mirror_url_download = config.get("Mirror", "url_download");

    // Memory
    if let Some(min) = config.get("Memory", "min") {
        app_config.min_memory = min.parse().unwrap_or(app_config.min_memory);
    }
    if let Some(max) = config.get("Memory", "max") {
        app_config.max_memory = max.parse().unwrap_or(app_config.max_memory);
    }

    // 自动模式：内存值为0时，根据系统内存动态计算
    if app_config.min_memory == 0 || app_config.max_memory == 0 {
        let sys_mem = crate::minecraft::system::get_system_memory();
        let available_mb = (sys_mem.available / 1024 / 1024) as u32;
        let suggested_max = std::cmp::min((available_mb as f64 * 0.75) as u32, 8192);
        let suggested_max = std::cmp::max(suggested_max, 512);
        let suggested_min = suggested_max / 2;

        if app_config.max_memory == 0 {
            app_config.max_memory = suggested_max;
        }
        if app_config.min_memory == 0 {
            app_config.min_memory = suggested_min;
        }
        log_info!("Auto memory config: min={}MB, max={}MB", app_config.min_memory, app_config.max_memory);
    }

    // Log
    if let Some(level) = config.get("Log", "level") {
        app_config.log_level = level.parse().unwrap_or(app_config.log_level);
    }

    // Proxy
    app_config.proxy_mode = config.get_or("Proxy", "mode", &app_config.proxy_mode);
    app_config.proxy_type = config.get_or("Proxy", "type", &app_config.proxy_type);
    app_config.proxy_url = config.get_or("Proxy", "url", &app_config.proxy_url);

    log_info!("Config loaded from storage");
    Ok(Some(app_config))
}

/// 保存配置到 storage
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let storage = Storage::instance();

    let mut ini = storage.read_config().unwrap_or_default();

    // General
    ini.set("General", "game_dir", &config.game_dir);
    ini.set("General", "theme", &config.theme);
    ini.set("General", "language", &config.language);

    // Download
    ini.set("Download", "max_threads", &config.max_download_threads.to_string());
    ini.set("Download", "max_speed", &config.max_download_speed.to_string());
    ini.set("Download", "source", &config.download_source);
    ini.set("Download", "mirror_mode", &config.mirror_mode.to_string());

    // Mirror
    if let Some(ref url) = config.mirror_url {
        ini.set("Mirror", "url", url);
    } else {
        ini.remove("Mirror", "url");
    }
    if let Some(ref url) = config.mirror_url_meta {
        ini.set("Mirror", "url_meta", url);
    } else {
        ini.remove("Mirror", "url_meta");
    }
    if let Some(ref url) = config.mirror_url_download {
        ini.set("Mirror", "url_download", url);
    } else {
        ini.remove("Mirror", "url_download");
    }

    // Memory
    ini.set("Memory", "min", &config.min_memory.to_string());
    ini.set("Memory", "max", &config.max_memory.to_string());

    // Log
    ini.set("Log", "level", &config.log_level.to_string());

    // Proxy
    ini.set("Proxy", "mode", &config.proxy_mode);
    ini.set("Proxy", "type", &config.proxy_type);
    ini.set("Proxy", "url", &config.proxy_url);

    storage.write_config(&ini).map_err(|e| e.to_string())?;
    log_debug!("Config saved to storage");
    Ok(())
}
