//! 配置默认值。

use super::models::{
    AppConfig, CommunityConfig, DownloadConfig, LaunchAdvancedConfig, McFolder, MemoryConfig,
    ProxyConfig, TlsConfig,
};

pub(crate) fn default_trust_mode() -> String {
    "builtin".to_string()
}

pub(crate) fn default_close_behavior() -> String {
    "ask".to_string()
}

/// OnlineConfig.easytier_core_path 默认值：externalBin 侧车位于 resource_dir 下 sidecar/ 目录
pub(crate) fn default_easytier_core_path() -> String {
    "sidecar/easytier-core.exe".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        let game_dir = super::paths::get_default_game_dir();
        Self {
            game_dir: game_dir.clone(),
            mc_folders: vec![McFolder {
                name: "默认".to_string(),
                path: game_dir,
            }],
            isolation_mode: 4,
            log_level: 3,
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            game_language: "zh_cn".to_string(),
            primary_color: "#165dff".to_string(),
            selected_version: None,
            external_download_dir: None,
            close_behavior: default_close_behavior(),
            experimental_enabled: false,
            proxy: ProxyConfig {
                mode: "none".to_string(),
                kind: "http".to_string(),
                url: String::new(),
                ip_version: "any".to_string(),
            },
            download: DownloadConfig {
                source: "smart".to_string(),
                meta_source: "smart".to_string(),
                max_speed: 0,
                max_threads: 8,
                chunk_count: 4,
                mirror_url: None,
                mirror_url_meta: None,
                mirror_url_download: None,
                mirror_mode: 0,
                modrinth_cdn_raw_enabled: false,
            },
            memory: MemoryConfig {
                mode: "auto".to_string(),
                min: 0,
                max: 0,
            },
            community: CommunityConfig {
                source: 2,
                filename_format: 1,
                mod_local_name_style: 0,
                ignore_quilt: true,
            },
            launch_advanced: LaunchAdvancedConfig {
                disable_jlw: false,
                disable_lua: false,
                use_dedicated_gpu: false,
            },
            online: super::models::OnlineConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}
