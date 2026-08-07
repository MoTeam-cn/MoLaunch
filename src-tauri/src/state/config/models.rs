//! 配置结构体与 serde 形状。

use crate::minecraft::online::signaling::IceServerEntry;
use serde::{Deserialize, Serialize};

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyConfig {
    pub mode: String,
    pub kind: String,
    pub url: String,
    #[serde(default)]
    pub ip_version: String,
}

/// 下载配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadConfig {
    pub source: String,
    pub meta_source: String,
    pub max_speed: u64,
    pub max_threads: u32,
    pub chunk_count: u32,
    pub mirror_url: Option<String>,
    pub mirror_url_meta: Option<String>,
    pub mirror_url_download: Option<String>,
    pub mirror_mode: u32,
    #[serde(default)]
    pub modrinth_cdn_raw_enabled: bool,
}

/// 内存配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    pub mode: String,
    pub min: u32,
    pub max: u32,
}

/// 社区资源配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommunityConfig {
    pub source: u8,
    pub filename_format: u8,
    pub mod_local_name_style: u8,
    pub ignore_quilt: bool,
}

/// 启动高级选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchAdvancedConfig {
    pub disable_jlw: bool,
    pub disable_lua: bool,
    pub use_dedicated_gpu: bool,
}

/// 联机功能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineConfig {
    pub api_server_url: String,
    #[serde(default)]
    pub custom_turn_servers: Vec<IceServerEntry>,
}

impl Default for OnlineConfig {
    fn default() -> Self {
        Self {
            api_server_url: "https://api.molaunch.moiu.cn".to_string(),
            custom_turn_servers: Vec::new(),
        }
    }
}

/// TLS 证书配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    #[serde(default = "crate::state::config::defaults::default_trust_mode")]
    pub trust_mode: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            trust_mode: crate::state::config::defaults::default_trust_mode(),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub game_dir: String,
    pub mc_folders: Vec<McFolder>,
    pub isolation_mode: u32,
    pub log_level: u32,
    pub theme: String,
    pub language: String,
    pub game_language: String,
    pub primary_color: String,
    pub selected_version: Option<String>,
    pub external_download_dir: Option<String>,
    #[serde(default = "crate::state::config::defaults::default_close_behavior")]
    pub close_behavior: String,
    #[serde(default)]
    pub experimental_enabled: bool,
    pub proxy: ProxyConfig,
    pub download: DownloadConfig,
    pub memory: MemoryConfig,
    pub community: CommunityConfig,
    pub launch_advanced: LaunchAdvancedConfig,
    pub online: OnlineConfig,
    pub tls: TlsConfig,
}

/// Minecraft 文件夹项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McFolder {
    pub name: String,
    pub path: String,
}
