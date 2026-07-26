//! 配置持久化模块
//!
//! 使用 storage 模块管理配置文件（INI 格式）

use crate::error_util::log_err;
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
    app_config.game_language = config.get_or("General", "game_language", &app_config.game_language);
    app_config.primary_color = config.get_or("General", "primary_color", &app_config.primary_color);

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
        app_config.download.max_threads =
            threads.parse().unwrap_or(app_config.download.max_threads);
    }
    if let Some(chunks) = config.get("Download", "chunk_count") {
        app_config.download.chunk_count = chunks.parse().unwrap_or(app_config.download.chunk_count);
    }
    if let Some(mode) = config.get("General", "isolation_mode") {
        app_config.isolation_mode = mode.parse().unwrap_or(app_config.isolation_mode);
    }
    if let Some(speed) = config.get("Download", "max_speed") {
        app_config.download.max_speed = speed.parse().unwrap_or(app_config.download.max_speed);
    }
    app_config.download.source =
        config.get_or("Download", "source", &app_config.download.source);
    app_config.download.meta_source =
        config.get_or("Download", "meta_source", &app_config.download.meta_source);
    if let Some(mode) = config.get("Download", "mirror_mode") {
        app_config.download.mirror_mode = mode.parse().unwrap_or(app_config.download.mirror_mode);
    }

    // Mirror
    app_config.download.mirror_url = config.get("Mirror", "url");
    app_config.download.mirror_url_meta = config.get("Mirror", "url_meta");
    app_config.download.mirror_url_download = config.get("Mirror", "url_download");

    // Memory
    app_config.memory.mode = config.get_or("Memory", "mode", &app_config.memory.mode);
    if let Some(min) = config.get("Memory", "min") {
        app_config.memory.min = min.parse().unwrap_or(app_config.memory.min);
    }
    if let Some(max) = config.get("Memory", "max") {
        app_config.memory.max = max.parse().unwrap_or(app_config.memory.max);
    }

    // 兼容旧配置：如果没有 mode 字段但有具体的内存值，则为自定义模式
    if config.get("Memory", "mode").is_none()
        && app_config.memory.min > 0
        && app_config.memory.max > 0
    {
        app_config.memory.mode = "custom".to_string();
        log_info!(
            "Legacy config detected: memory mode set to custom (min={}MB, max={}MB)",
            app_config.memory.min,
            app_config.memory.max
        );
    }

    // 自动模式：如果 memory_mode 为 "auto"，计算自动值用于运行时
    if app_config.memory.mode == "auto" {
        let (suggested_min, suggested_max) = crate::minecraft::system::suggest_memory();
        app_config.memory.min = suggested_min;
        app_config.memory.max = suggested_max;
        log_info!(
            "Auto memory config: min={}MB, max={}MB",
            app_config.memory.min,
            app_config.memory.max
        );
    }

    // Log
    if let Some(level) = config.get("Log", "level") {
        app_config.log_level = level.parse().unwrap_or(app_config.log_level);
    }

    // Proxy
    app_config.proxy.mode = config.get_or("Proxy", "mode", &app_config.proxy.mode);
    app_config.proxy.kind = config.get_or("Proxy", "type", &app_config.proxy.kind);
    app_config.proxy.url = config.get_or("Proxy", "url", &app_config.proxy.url);

    // Community（社区资源配置）
    if let Some(v) = config.get("Community", "source") {
        app_config.community.source = v.parse().unwrap_or(app_config.community.source);
    }
    if let Some(v) = config.get("Community", "filename_format") {
        app_config.community.filename_format =
            v.parse().unwrap_or(app_config.community.filename_format);
    }
    if let Some(v) = config.get("Community", "mod_local_name_style") {
        app_config.community.mod_local_name_style = v
            .parse()
            .unwrap_or(app_config.community.mod_local_name_style);
    }
    if let Some(v) = config.get("Community", "ignore_quilt") {
        app_config.community.ignore_quilt = v == "true" || v == "1";
    }

    // Launch（启动高级选项）
    if let Some(v) = config.get("Launch", "disable_jlw") {
        app_config.launch_advanced.disable_jlw = v == "true" || v == "1";
    }
    if let Some(v) = config.get("Launch", "disable_lua") {
        app_config.launch_advanced.disable_lua = v == "true" || v == "1";
    }
    if let Some(v) = config.get("Launch", "use_dedicated_gpu") {
        app_config.launch_advanced.use_dedicated_gpu = v == "true" || v == "1";
    }

    // ExternalDownload
    app_config.external_download_dir = config
        .get("ExternalDownload", "dir")
        .filter(|s| !s.is_empty())
        .map(|s| s.clone());

    // Version
    app_config.selected_version = config
        .get("Version", "selected")
        .filter(|s| !s.is_empty())
        .map(|s| s.clone());

    // Online（联机 api-server 地址，未配置时保留默认值）
    if let Some(url) = config.get("Online", "api_server_url") {
        if !url.is_empty() {
            app_config.online.api_server_url = url;
        }
    }

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
    ini.set("General", "game_language", &config.game_language);
    ini.set("General", "primary_color", &config.primary_color);
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
        &config.download.max_threads.to_string(),
    );
    ini.set(
        "Download",
        "chunk_count",
        &config.download.chunk_count.to_string(),
    );
    ini.set(
        "Download",
        "max_speed",
        &config.download.max_speed.to_string(),
    );
    ini.set("Download", "source", &config.download.source);
    ini.set("Download", "meta_source", &config.download.meta_source);
    ini.set(
        "Download",
        "mirror_mode",
        &config.download.mirror_mode.to_string(),
    );

    // Mirror
    if let Some(ref url) = config.download.mirror_url {
        ini.set("Mirror", "url", url);
    } else {
        ini.remove("Mirror", "url");
    }
    if let Some(ref url) = config.download.mirror_url_meta {
        ini.set("Mirror", "url_meta", url);
    } else {
        ini.remove("Mirror", "url_meta");
    }
    if let Some(ref url) = config.download.mirror_url_download {
        ini.set("Mirror", "url_download", url);
    } else {
        ini.remove("Mirror", "url_download");
    }

    // Memory
    ini.set("Memory", "mode", &config.memory.mode);
    if config.memory.mode == "custom" {
        ini.set("Memory", "min", &config.memory.min.to_string());
        ini.set("Memory", "max", &config.memory.max.to_string());
    } else {
        // 自动模式下不保存具体值，只保存 mode
        ini.remove("Memory", "min");
        ini.remove("Memory", "max");
    }

    // Log
    ini.set("Log", "level", &config.log_level.to_string());

    // Proxy
    ini.set("Proxy", "mode", &config.proxy.mode);
    ini.set("Proxy", "type", &config.proxy.kind);
    ini.set("Proxy", "url", &config.proxy.url);

    // Community
    ini.set(
        "Community",
        "source",
        &config.community.source.to_string(),
    );
    ini.set(
        "Community",
        "filename_format",
        &config.community.filename_format.to_string(),
    );
    ini.set(
        "Community",
        "mod_local_name_style",
        &config.community.mod_local_name_style.to_string(),
    );
    ini.set(
        "Community",
        "ignore_quilt",
        if config.community.ignore_quilt {
            "true"
        } else {
            "false"
        },
    );

    // Launch（启动高级选项）
    ini.set(
        "Launch",
        "disable_jlw",
        if config.launch_advanced.disable_jlw {
            "true"
        } else {
            "false"
        },
    );
    ini.set(
        "Launch",
        "disable_lua",
        if config.launch_advanced.disable_lua {
            "true"
        } else {
            "false"
        },
    );
    ini.set(
        "Launch",
        "use_dedicated_gpu",
        if config.launch_advanced.use_dedicated_gpu {
            "true"
        } else {
            "false"
        },
    );

    // Version
    ini.set(
        "Version",
        "selected",
        config.selected_version.as_deref().unwrap_or(""),
    );

    // ExternalDownload
    ini.set(
        "ExternalDownload",
        "dir",
        config.external_download_dir.as_deref().unwrap_or(""),
    );

    // Online
    ini.set("Online", "api_server_url", &config.online.api_server_url);

    storage.write_config(&ini).map_err(log_err("Failed to save config"))?;
    log_debug!("Config saved to storage");
    Ok(())
}
