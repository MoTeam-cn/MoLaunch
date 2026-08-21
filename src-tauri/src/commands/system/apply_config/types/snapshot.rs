//! 配置快照类型：get_config 返回的全量配置快照

use crate::minecraft::online::signaling::IceServerEntry;
use crate::utils::github_download::GithubProxy;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OnlineSnapshot {
    #[serde(rename = "onlineApiServerUrl")]
    pub api_server_url: String,
    /// 用户自定义 TURN 服务器列表（阶段三子任务 7 新增）
    #[serde(rename = "onlineCustomTurnServers", default)]
    pub custom_turn_servers: Vec<IceServerEntry>,
    /// 公共 easytier 中继节点列表（`--peers` 参数）
    #[serde(rename = "onlineEasytierPublicPeers", default)]
    pub easytier_public_peers: Vec<String>,
    /// 用户自定义 GitHub 镜像源（full/path 模式）
    #[serde(rename = "onlineGithubProxies", default)]
    pub github_proxies: Vec<GithubProxy>,
}

/// TLS 配置快照（serde(flatten) 展平到 ConfigSnapshot）
///
/// - `trust_mode`：来自 `AppConfig.tls.trust_mode`（INI 持久化）
/// - `ignore_tls`：来自注册表 `IgnoreTls`（开发者模式键，不进 AppConfig）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSnapshot {
    #[serde(rename = "tlsTrustMode")]
    pub trust_mode: String,
    #[serde(rename = "ignoreTls")]
    pub ignore_tls: bool,
}

impl Default for TlsSnapshot {
    fn default() -> Self {
        Self {
            trust_mode: "builtin".to_string(),
            ignore_tls: false,
        }
    }
}

/// 关闭主窗口时的默认行为（每次询问）
fn default_close_behavior_str() -> String {
    "ask".to_string()
}

/// GPU 硬件加速默认开启
fn default_use_gpu_acceleration() -> bool {
    true
}

/// 关闭到托盘时挂起 WebView2 默认关闭
fn default_release_memory_on_tray() -> bool {
    false
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
    // 通用字段（保持平铺，rename_all = "camelCase" 自动映射）
    pub game_dir: String,
    pub isolation_mode: u32,
    pub log_level: u32,
    /// 游戏默认界面语言
    pub game_language: String,
    /// 主题主色 HEX（如 "#165dff"）
    pub primary_color: String,
    /// 关闭主窗口时的行为："ask"（每次询问）/ "tray"（保留托盘）/ "exit"（直接退出）
    #[serde(default = "default_close_behavior_str")]
    pub close_behavior: String,
    /// 实验性功能开关（开启后显示「实验性」入口并初始化 SQLite 聊天存储，默认 false）
    pub experimental_enabled: bool,
    /// 启动器界面 GPU 硬件加速（默认开启；关闭后 WebView2 走软件渲染）
    #[serde(default = "default_use_gpu_acceleration")]
    pub use_gpu_acceleration: bool,
    /// 关闭到托盘时挂起 WebView2 释放渲染资源（默认关闭）
    #[serde(default = "default_release_memory_on_tray")]
    pub release_memory_on_tray: bool,
    pub selected_version: Option<String>,
    // 外部下载工具
    pub external_download_dir: Option<String>,
    /// Java 路径（从 INI [Java] path 读取，不进 AppConfig）
    #[serde(default)]
    pub java_path: Option<String>,

    // 分组字段（serde(flatten) 展平到顶层）
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
    #[serde(flatten)]
    pub tls: TlsSnapshot,

    // CurseForge（从 secure_storage 缓存读，已解密）
    pub curseforge_enabled: bool,
    pub curseforge_api_key: String,
    // 开发者模式（从注册表读）
    pub developer_unlocked: bool,
    pub developer_mode: bool,
    // 正版购买提示（从系统存储读：Windows 注册表 / 其他系统全局共用文件）
    #[serde(default)]
    pub launch_count: u32,
    #[serde(default)]
    pub hint_buy: bool,
    #[serde(default)]
    pub hint_star: bool,
    // 用户协议（系统存储：Windows 注册表 / 其他系统全局共用文件，全局首次启动门禁）
    /// 是否已同意《用户协议》
    #[serde(default)]
    pub user_agreed: bool,
    /// 已同意的《用户协议》版本号（0 表示从未同意）
    #[serde(default)]
    pub user_agreed_version: u32,
}
