//! 配置数据类型：补丁、快照、条目
//!
//! - `ConfigPatch`：`apply_config` 入参，所有字段 `Option<T>`，仅传需要改的字段
//! - `ConfigSnapshot`：`get_config` 返回的全量配置快照
//! - `ConfigEntry`：扁平化 key-value 对，前后端 IPC 格式对称

use crate::minecraft::online::signaling::IceServerEntry;
use serde::{Deserialize, Serialize};

// ============================================================
// ConfigPatch 子 struct（serde(flatten) 展平到 ConfigPatch）
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyPatch {
    #[serde(rename = "proxyMode")]
    pub mode: Option<String>,
    #[serde(rename = "proxyType")]
    pub kind: Option<String>,
    #[serde(rename = "proxyUrl")]
    pub url: Option<String>,
    #[serde(rename = "ipVersion")]
    pub ip_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadPatch {
    #[serde(rename = "downloadSource")]
    pub source: Option<String>,
    #[serde(rename = "metaSource")]
    pub meta_source: Option<String>,
    #[serde(rename = "maxDownloadSpeed")]
    pub max_speed: Option<u64>,
    #[serde(rename = "maxDownloadThreads")]
    pub max_threads: Option<u32>,
    #[serde(rename = "chunkCount")]
    pub chunk_count: Option<u32>,
    /// 双层 Option：外层 Some 表示"要更新此字段"，内层 None 表示"清空"
    #[serde(rename = "mirrorUrl")]
    pub mirror_url: Option<Option<String>>,
    /// Modrinth CDN 直连开关（开发者模式可见，默认 false）
    #[serde(rename = "modrinthCdnRawEnabled")]
    pub modrinth_cdn_raw_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryPatch {
    #[serde(rename = "memoryMode")]
    pub mode: Option<String>,
    #[serde(rename = "minMemory")]
    pub min: Option<u32>,
    #[serde(rename = "maxMemory")]
    pub max: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommunityPatch {
    #[serde(rename = "communitySource")]
    pub source: Option<u8>,
    #[serde(rename = "communityFilenameFormat")]
    pub filename_format: Option<u8>,
    #[serde(rename = "communityModLocalNameStyle")]
    pub mod_local_name_style: Option<u8>,
    #[serde(rename = "communityIgnoreQuilt")]
    pub ignore_quilt: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchAdvancedPatch {
    #[serde(rename = "launchDisableJlw")]
    pub disable_jlw: Option<bool>,
    #[serde(rename = "launchDisableLua")]
    pub disable_lua: Option<bool>,
    #[serde(rename = "launchUseDedicatedGpu")]
    pub use_dedicated_gpu: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OnlinePatch {
    #[serde(rename = "onlineApiServerUrl")]
    pub api_server_url: Option<String>,
    /// 用户自定义 TURN 服务器列表（阶段三子任务 7 新增）
    ///
    /// `Some` 表示要更新此字段（含空数组，表示清空所有自定义 TURN）；
    /// `None` 表示不更新。
    #[serde(rename = "onlineCustomTurnServers")]
    pub custom_turn_servers: Option<Vec<IceServerEntry>>,
}

/// 配置补丁：所有字段可选，仅传需要更新的字段
///
/// 字段命名采用 camelCase 序列化（前端约定），与 `AppConfig` 的 snake_case
/// 字段一一对应。通用字段通过 `#[serde(rename_all = "camelCase")]` 自动映射，
/// 分组字段通过子 struct 的 `#[serde(rename = "...")]` 显式指定，并使用
/// `#[serde(flatten)]` 展平到顶层 JSON（保持前端 `Vec<ConfigEntry>` 的 flat key 不变）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPatch {
    // ===== 通用字段（保持平铺，rename_all = "camelCase" 自动映射）=====
    pub game_dir: Option<String>,
    pub isolation_mode: Option<u32>,
    pub log_level: Option<u32>,
    /// 游戏默认界面语言：写入 options.txt 的 lang 字段
    /// - "auto"：跟随启动器语言（旧配置兼容）
    /// - "zh_cn" / "en_us" / "ja_jp" / "ko_kr" 等 MC 标准语言代码
    /// - "none"：不设置
    pub game_language: Option<String>,
    /// 主题主色 HEX（如 "#165dff"），前端注入 CSS 变量驱动 primary-* 色阶
    pub primary_color: Option<String>,
    /// 双层 Option：外层 Some 表示"要更新此字段"，内层 None 表示"清空"
    pub selected_version: Option<Option<String>>,
    /// 双层 Option：外层 Some 表示"要更新此字段"，内层 None 表示"清空（回退默认目录）"
    pub external_download_dir: Option<Option<String>>,

    // ===== 分组字段（serde(flatten) 展平到顶层）=====
    #[serde(flatten)]
    pub proxy: ProxyPatch,
    #[serde(flatten)]
    pub download: DownloadPatch,
    #[serde(flatten)]
    pub memory: MemoryPatch,
    #[serde(flatten)]
    pub community: CommunityPatch,
    #[serde(flatten)]
    pub launch_advanced: LaunchAdvancedPatch,
    #[serde(flatten)]
    pub online: OnlinePatch,

    // ===== CurseForge（加密存储，不进 AppConfig，内部分流到 secure_storage）=====
    pub curseforge_enabled: Option<bool>,
    pub curseforge_api_key: Option<String>,

    // ===== 开发者模式（注册表存储，不进 AppConfig，内部分流到 registry）=====
    /// 开关是否开启（仅在已解锁时可生效）
    pub developer_mode: Option<bool>,
}

// ============================================================
// ConfigSnapshot 子 struct（serde(flatten) 展平到 ConfigSnapshot）
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxySnapshot {
    #[serde(rename = "proxyMode")]
    pub mode: String,
    #[serde(rename = "proxyType")]
    pub kind: String,
    #[serde(rename = "proxyUrl")]
    pub url: String,
    #[serde(rename = "ipVersion")]
    pub ip_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadSnapshot {
    #[serde(rename = "mirrorUrl")]
    pub mirror_url: Option<String>,
    #[serde(rename = "downloadSource")]
    pub source: String,
    #[serde(rename = "metaSource")]
    pub meta_source: String,
    #[serde(rename = "maxDownloadSpeed")]
    pub max_speed: u64,
    #[serde(rename = "maxDownloadThreads")]
    pub max_threads: u32,
    #[serde(rename = "chunkCount")]
    pub chunk_count: u32,
    /// Modrinth CDN 直连开关（开发者模式可见，默认 false）
    #[serde(rename = "modrinthCdnRawEnabled")]
    pub modrinth_cdn_raw_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySnapshot {
    #[serde(rename = "memoryMode")]
    pub mode: String,
    #[serde(rename = "minMemory")]
    pub min: u32,
    #[serde(rename = "maxMemory")]
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommunitySnapshot {
    #[serde(rename = "communitySource")]
    pub source: u8,
    #[serde(rename = "communityFilenameFormat")]
    pub filename_format: u8,
    #[serde(rename = "communityModLocalNameStyle")]
    pub mod_local_name_style: u8,
    #[serde(rename = "communityIgnoreQuilt")]
    pub ignore_quilt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchAdvancedSnapshot {
    #[serde(rename = "launchDisableJlw")]
    pub disable_jlw: bool,
    #[serde(rename = "launchDisableLua")]
    pub disable_lua: bool,
    #[serde(rename = "launchUseDedicatedGpu")]
    pub use_dedicated_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineSnapshot {
    #[serde(rename = "onlineApiServerUrl")]
    pub api_server_url: String,
    /// 用户自定义 TURN 服务器列表（阶段三子任务 7 新增）
    #[serde(rename = "onlineCustomTurnServers", default)]
    pub custom_turn_servers: Vec<IceServerEntry>,
}

impl Default for OnlineSnapshot {
    fn default() -> Self {
        Self {
            api_server_url: String::new(),
            custom_turn_servers: Vec::new(),
        }
    }
}

/// 配置快照：返回所有配置字段的当前值
///
/// 用于前端一次性读取全部配置，取代此前分散的 14 个 get_* 命令。
/// CurseForge 的 api_key 从 secure_storage 缓存读取（已解密），
/// 若首次未解密则返回空字符串（懒加载，避免触发杀软误报）。
/// 开发者模式从注册表读取（DeveloperUnlocked / DeveloperMode）。
///
/// 通用字段通过 `#[serde(rename_all = "camelCase")]` 自动映射，
/// 分组字段通过子 struct 的 `#[serde(rename = "...")]` 显式指定，并使用
/// `#[serde(flatten)]` 展平到顶层 JSON（保持前端 `Vec<ConfigEntry>` 的 flat key 不变）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSnapshot {
    // ===== 通用字段（保持平铺，rename_all = "camelCase" 自动映射）=====
    pub game_dir: String,
    pub isolation_mode: u32,
    pub log_level: u32,
    /// 游戏默认界面语言
    pub game_language: String,
    /// 主题主色 HEX（如 "#165dff"）
    pub primary_color: String,
    pub selected_version: Option<String>,
    // 外部下载工具
    pub external_download_dir: Option<String>,

    // ===== 分组字段（serde(flatten) 展平到顶层）=====
    #[serde(flatten)]
    pub proxy: ProxySnapshot,
    #[serde(flatten)]
    pub download: DownloadSnapshot,
    #[serde(flatten)]
    pub memory: MemorySnapshot,
    #[serde(flatten)]
    pub community: CommunitySnapshot,
    #[serde(flatten)]
    pub launch_advanced: LaunchAdvancedSnapshot,
    #[serde(flatten)]
    pub online: OnlineSnapshot,

    // ===== CurseForge（从 secure_storage 缓存读，已解密）=====
    pub curseforge_enabled: bool,
    pub curseforge_api_key: String,
    // 开发者模式（从注册表读）
    pub developer_unlocked: bool,
    pub developer_mode: bool,
}

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
/// CurseForge / 开发者模式字段由调用方提前读取（分别在 secure_storage 缓存与
/// 注册表中），其余字段从 `AppConfig` 镜像。集中在此处避免散落在命令实现里，
/// 也使 `get_config` 命令体保持简短。
pub fn build_snapshot(
    config: &crate::state::AppConfig,
    cf_enabled: bool,
    cf_api_key: Option<String>,
    dev_unlocked: bool,
    dev_mode: bool,
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

        // 非 AppConfig 字段
        curseforge_enabled: cf_enabled,
        curseforge_api_key: cf_api_key.unwrap_or_default(),
        developer_unlocked: dev_unlocked,
        developer_mode: dev_mode,
    }
}
