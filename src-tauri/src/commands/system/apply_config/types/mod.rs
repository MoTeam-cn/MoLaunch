//! 配置数据类型：补丁、快照、条目
//!
//! - `ConfigPatch`：`apply_config` 入参，所有字段 `Option<T>`，仅传需要改的字段
//! - `ConfigSnapshot`：`get_config` 返回的全量配置快照
//! - `ConfigEntry`：扁平化 key-value 对，前后端 IPC 格式对称

mod patch;
mod snapshot;

pub use patch::ConfigPatch;
pub use snapshot::ConfigSnapshot;

use serde::{Deserialize, Serialize};
use snapshot::{
    CommunitySnapshot, DownloadSnapshot, LaunchAdvancedSnapshot, MemorySnapshot, OnlineSnapshot,
    ProxySnapshot, TlsSnapshot,
};

/// 配置项：扁平化 key-value 对
///
/// `get_config` 返回 `Vec<ConfigEntry>`，`apply_config` 接收同样的 `Vec<ConfigEntry>`，
/// 前后端格式完全对称。每项形如 `{ "key": "proxyMode", "value": "none" }`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: serde_json::Value,
}

/// 从已锁定的 `AppConfig` 构建配置快照
///
/// CurseForge / 开发者模式 / IgnoreTls 字段由调用方提前读取（分别在 secure_storage 缓存与
/// 注册表中），其余字段从 `AppConfig` 镜像。集中在此处避免散落在命令实现里，
/// 也使 `get_config` 命令体保持简短。
pub fn build_snapshot(
    config: &crate::state::AppConfig,
    cf_enabled: bool,
    cf_api_key: Option<String>,
    dev_unlocked: bool,
    dev_mode: bool,
    ignore_tls: bool,
    java_path: Option<String>,
) -> ConfigSnapshot {
    ConfigSnapshot {
        // 通用字段
        game_dir: config.game_dir.clone(),
        isolation_mode: config.isolation_mode,
        log_level: config.log_level,
        game_language: config.game_language.clone(),
        primary_color: config.primary_color.clone(),
        selected_version: config.selected_version.clone(),
        external_download_dir: config.external_download_dir.clone(),
        java_path,

        // 分组字段
        proxy: ProxySnapshot {
            mode: config.proxy.mode.clone(),
            kind: config.proxy.kind.clone(),
            url: config.proxy.url.clone(),
            ip_version: config.proxy.ip_version.clone(),
        },
        download: DownloadSnapshot {
            mirror_url: config.download.mirror_url.clone(),
            source: config.download.source.clone(),
            meta_source: config.download.meta_source.clone(),
            max_speed: config.download.max_speed,
            max_threads: config.download.max_threads,
            chunk_count: config.download.chunk_count,
            modrinth_cdn_raw_enabled: config.download.modrinth_cdn_raw_enabled,
        },
        memory: MemorySnapshot {
            mode: config.memory.mode.clone(),
            min: config.memory.min,
            max: config.memory.max,
        },
        community: CommunitySnapshot {
            source: config.community.source,
            filename_format: config.community.filename_format,
            mod_local_name_style: config.community.mod_local_name_style,
            ignore_quilt: config.community.ignore_quilt,
        },
        launch_advanced: LaunchAdvancedSnapshot {
            disable_jlw: config.launch_advanced.disable_jlw,
            disable_lua: config.launch_advanced.disable_lua,
            use_dedicated_gpu: config.launch_advanced.use_dedicated_gpu,
        },
        online: OnlineSnapshot {
            api_server_url: config.online.api_server_url.clone(),
            custom_turn_servers: config.online.custom_turn_servers.clone(),
        },
        tls: TlsSnapshot {
            trust_mode: config.tls.trust_mode.clone(),
            ignore_tls,
        },

        // 非 AppConfig 字段
        curseforge_enabled: cf_enabled,
        curseforge_api_key: cf_api_key.unwrap_or_default(),
        developer_unlocked: dev_unlocked,
        developer_mode: dev_mode,
    }
}
