//! 配置持久化模块
//!
//! 使用 storage 模块管理配置文件（INI 格式）

use crate::state::AppConfig;
use crate::storage::Storage;
use crate::{log_debug, log_info, log_warn};

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

    // Folders（Minecraft 文件夹列表）
    if let Some(list_json) = config.get("Folders", "list") {
        match serde_json::from_str::<Vec<crate::state::McFolder>>(&list_json) {
            Ok(folders) => {
                if !folders.is_empty() {
                    app_config.mc_folders = folders;
                }
            }
            Err(e) => {
                log_warn!("Failed to parse mc_folders list: {}", e);
            }
        }
    }
    // 兼容：如果 mc_folders 为空但 game_dir 存在，用 game_dir 作为默认文件夹
    if app_config.mc_folders.is_empty() {
        app_config.mc_folders = vec![crate::state::McFolder {
            name: "默认".to_string(),
            path: app_config.game_dir.clone(),
        }];
    }

    // Download
    if let Some(threads) = config.get("Download", "max_threads") {
        app_config.max_download_threads =
            threads.parse().unwrap_or(app_config.max_download_threads);
    }
    if let Some(chunks) = config.get("Download", "chunk_count") {
        app_config.chunk_count = chunks.parse().unwrap_or(app_config.chunk_count);
    }
    if let Some(mode) = config.get("General", "isolation_mode") {
        app_config.isolation_mode = mode.parse().unwrap_or(app_config.isolation_mode);
    }
    if let Some(speed) = config.get("Download", "max_speed") {
        app_config.max_download_speed = speed.parse().unwrap_or(app_config.max_download_speed);
    }
    app_config.download_source = config.get_or("Download", "source", &app_config.download_source);
    app_config.meta_source = config.get_or("Download", "meta_source", &app_config.meta_source);
    if let Some(mode) = config.get("Download", "mirror_mode") {
        app_config.mirror_mode = mode.parse().unwrap_or(app_config.mirror_mode);
    }

    // Mirror
    app_config.mirror_url = config.get("Mirror", "url");
    app_config.mirror_url_meta = config.get("Mirror", "url_meta");
    app_config.mirror_url_download = config.get("Mirror", "url_download");

    // Memory
    app_config.memory_mode = config.get_or("Memory", "mode", &app_config.memory_mode);
    if let Some(min) = config.get("Memory", "min") {
        app_config.min_memory = min.parse().unwrap_or(app_config.min_memory);
    }
    if let Some(max) = config.get("Memory", "max") {
        app_config.max_memory = max.parse().unwrap_or(app_config.max_memory);
    }

    // 兼容旧配置：如果没有 mode 字段但有具体的内存值，则为自定义模式
    if config.get("Memory", "mode").is_none()
        && app_config.min_memory > 0
        && app_config.max_memory > 0
    {
        app_config.memory_mode = "custom".to_string();
        log_info!(
            "Legacy config detected: memory mode set to custom (min={}MB, max={}MB)",
            app_config.min_memory,
            app_config.max_memory
        );
    }

    // 自动模式：如果 memory_mode 为 "auto"，计算自动值用于运行时
    if app_config.memory_mode == "auto" {
        let (suggested_min, suggested_max) =
            crate::minecraft::system::suggest_memory();
        app_config.min_memory = suggested_min;
        app_config.max_memory = suggested_max;
        log_info!(
            "Auto memory config: min={}MB, max={}MB",
            app_config.min_memory,
            app_config.max_memory
        );
    }

    // Log
    if let Some(level) = config.get("Log", "level") {
        app_config.log_level = level.parse().unwrap_or(app_config.log_level);
    }

    // Proxy
    app_config.proxy_mode = config.get_or("Proxy", "mode", &app_config.proxy_mode);
    app_config.proxy_type = config.get_or("Proxy", "type", &app_config.proxy_type);
    app_config.proxy_url = config.get_or("Proxy", "url", &app_config.proxy_url);

    // Community（社区资源配置，参考 PCL2 PageSetupSystem "社区资源" 卡片）
    if let Some(v) = config.get("Community", "source") {
        app_config.community_source = v.parse().unwrap_or(app_config.community_source);
    }
    if let Some(v) = config.get("Community", "filename_format") {
        app_config.community_filename_format = v.parse().unwrap_or(app_config.community_filename_format);
    }
    if let Some(v) = config.get("Community", "mod_local_name_style") {
        app_config.community_mod_local_name_style = v.parse().unwrap_or(app_config.community_mod_local_name_style);
    }
    if let Some(v) = config.get("Community", "ignore_quilt") {
        app_config.community_ignore_quilt = v == "true" || v == "1";
    }

    // Launch（启动高级选项，参考 PCL2 PageSetupLaunch 高级选项）
    if let Some(v) = config.get("Launch", "disable_jlw") {
        app_config.launch_disable_jlw = v == "true" || v == "1";
    }
    if let Some(v) = config.get("Launch", "disable_lua") {
        app_config.launch_disable_lua = v == "true" || v == "1";
    }
    if let Some(v) = config.get("Launch", "use_dedicated_gpu") {
        app_config.launch_use_dedicated_gpu = v == "true" || v == "1";
    }

    // Version
    app_config.selected_version = config
        .get("Version", "selected")
        .filter(|s| !s.is_empty())
        .map(|s| s.clone());

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
    ini.set(
        "General",
        "isolation_mode",
        &config.isolation_mode.to_string(),
    );

    // Folders（Minecraft 文件夹列表，JSON 序列化存储）
    ini.set(
        "Folders",
        "list",
        &serde_json::to_string(&config.mc_folders).unwrap_or_else(|_| "[]".to_string()),
    );

    // Download
    ini.set(
        "Download",
        "max_threads",
        &config.max_download_threads.to_string(),
    );
    ini.set("Download", "chunk_count", &config.chunk_count.to_string());
    ini.set(
        "Download",
        "max_speed",
        &config.max_download_speed.to_string(),
    );
    ini.set("Download", "source", &config.download_source);
    ini.set("Download", "meta_source", &config.meta_source);
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
    ini.set("Memory", "mode", &config.memory_mode);
    if config.memory_mode == "custom" {
        ini.set("Memory", "min", &config.min_memory.to_string());
        ini.set("Memory", "max", &config.max_memory.to_string());
    } else {
        // 自动模式下不保存具体值，只保存 mode
        ini.remove("Memory", "min");
        ini.remove("Memory", "max");
    }

    // Log
    ini.set("Log", "level", &config.log_level.to_string());

    // Proxy
    ini.set("Proxy", "mode", &config.proxy_mode);
    ini.set("Proxy", "type", &config.proxy_type);
    ini.set("Proxy", "url", &config.proxy_url);

    // Community
    ini.set("Community", "source", &config.community_source.to_string());
    ini.set("Community", "filename_format", &config.community_filename_format.to_string());
    ini.set("Community", "mod_local_name_style", &config.community_mod_local_name_style.to_string());
    ini.set("Community", "ignore_quilt", if config.community_ignore_quilt { "true" } else { "false" });

    // Launch（启动高级选项）
    ini.set("Launch", "disable_jlw", if config.launch_disable_jlw { "true" } else { "false" });
    ini.set("Launch", "disable_lua", if config.launch_disable_lua { "true" } else { "false" });
    ini.set("Launch", "use_dedicated_gpu", if config.launch_use_dedicated_gpu { "true" } else { "false" });

    // Version
    ini.set("Version", "selected", config.selected_version.as_deref().unwrap_or(""));

    storage.write_config(&ini).map_err(|e| e.to_string())?;
    log_debug!("Config saved to storage");
    Ok(())
}
