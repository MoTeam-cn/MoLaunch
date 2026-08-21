//! 配置读取：INI → AppConfig

use crate::state::AppConfig;
use crate::storage::Storage;
use crate::{log_info, log_warn};

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
    app_config.close_behavior =
        config.get_or("General", "close_behavior", &app_config.close_behavior);
    if let Some(v) = config.get("General", "use_gpu_acceleration") {
        app_config.use_gpu_acceleration = v == "true" || v == "1";
    }
    // 实验性功能开关（[Experimental] enabled；未配置时保持默认 false）
    app_config.experimental_enabled =
        config.get("Experimental", "enabled").as_deref() == Some("true");

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
    app_config.download.source = config.get_or("Download", "source", &app_config.download.source);
    app_config.download.meta_source =
        config.get_or("Download", "meta_source", &app_config.download.meta_source);
    if let Some(mode) = config.get("Download", "mirror_mode") {
        app_config.download.mirror_mode = mode.parse().unwrap_or(app_config.download.mirror_mode);
    }
    app_config.download.modrinth_cdn_raw_enabled = config
        .get("Download", "modrinth_cdn_raw_enabled")
        .as_deref()
        == Some("true");

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
    app_config.proxy.ip_version =
        config.get_or("Proxy", "ip_version", &app_config.proxy.ip_version);

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
        .filter(|s| !s.is_empty());

    // Version
    app_config.selected_version = config.get("Version", "selected").filter(|s| !s.is_empty());

    // Online（联机 api-server 地址，未配置时保留默认值）
    if let Some(url) = config.get("Online", "api_server_url") {
        if !url.is_empty() {
            app_config.online.api_server_url = url;
        }
    }

    // Online.github_proxies（GitHub 镜像源，JSON 序列化存储）
    if let Some(list_json) = config.get("Online", "github_proxies") {
        match serde_json::from_str::<Vec<crate::utils::github_download::GithubProxy>>(&list_json) {
            Ok(proxies) => {
                if !proxies.is_empty() {
                    app_config.online.github_proxies = proxies;
                }
            }
            Err(e) => {
                log_warn!("Failed to parse github_proxies list: {}", e);
            }
        }
    }

    // TLS（信任源模式，未配置时保留默认 builtin）
    app_config.tls.trust_mode = config.get_or("TLS", "trust_mode", &app_config.tls.trust_mode);

    log_info!("Config loaded from storage");
    Ok(Some(app_config))
}
