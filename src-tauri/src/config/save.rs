//! 配置保存：AppConfig → INI

use crate::error_util::log_err;
use crate::log_debug;
use crate::state::AppConfig;
use crate::storage::Storage;

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
    ini.set("General", "close_behavior", &config.close_behavior);
    // Experimental（实验性功能开关）
    ini.set(
        "Experimental",
        "enabled",
        if config.experimental_enabled { "true" } else { "false" },
    );
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
    ini.set(
        "Download",
        "modrinth_cdn_raw_enabled",
        if config.download.modrinth_cdn_raw_enabled {
            "true"
        } else {
            "false"
        },
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
    ini.set("Proxy", "ip_version", &config.proxy.ip_version);

    // Community
    ini.set("Community", "source", &config.community.source.to_string());
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

    // TLS（信任源模式持久化，IgnoreTls 走注册表不在此处）
    ini.set("TLS", "trust_mode", &config.tls.trust_mode);

    storage
        .write_config(&ini)
        .map_err(log_err("Failed to save config"))?;
    log_debug!("Config saved to storage");
    Ok(())
}
