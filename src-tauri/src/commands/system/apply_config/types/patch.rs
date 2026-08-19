//! 配置补丁类型：apply_config 入参，所有字段 Option<T>，仅传需要改的字段

use crate::minecraft::online::signaling::IceServerEntry;
use crate::utils::github_download::GithubProxy;
use serde::{Deserialize, Serialize};

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
    /// 公共 easytier 中继节点列表（`--peers` 参数）
    ///
    /// `Some` 表示要更新此字段（含空数组，表示清空）；`None` 表示不更新。
    #[serde(rename = "onlineEasytierPublicPeers")]
    pub easytier_public_peers: Option<Vec<String>>,
    /// 用户自定义 GitHub 镜像源（full/path 模式）
    ///
    /// `Some` 表示要更新此字段（含空数组，表示清空）；`None` 表示不更新。
    #[serde(rename = "onlineGithubProxies")]
    pub github_proxies: Option<Vec<GithubProxy>>,
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
    // 通用字段（保持平铺，rename_all = "camelCase" 自动映射）
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
    /// 关闭主窗口时的行为："ask"（每次询问）/ "tray"（保留托盘）/ "exit"（直接退出）
    pub close_behavior: Option<String>,
    /// 实验性功能开关（开启后显示「实验性」入口，并惰性初始化 SQLite 聊天存储）
    pub experimental_enabled: Option<bool>,
    /// 双层 Option：外层 Some 表示"要更新此字段"，内层 None 表示"清空"
    pub selected_version: Option<Option<String>>,
    /// 双层 Option：外层 Some 表示"要更新此字段"，内层 None 表示"清空（回退默认目录）"
    pub external_download_dir: Option<Option<String>>,
    /// Java 路径（独立存储于 INI [Java] path，不进 AppConfig）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_path: Option<String>,

    // 分组字段（serde(flatten) 展平到顶层）
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

    // CurseForge（加密存储，不进 AppConfig，内部分流到 secure_storage）
    pub curseforge_enabled: Option<bool>,
    pub curseforge_api_key: Option<String>,

    // 开发者模式（注册表存储，不进 AppConfig，内部分流到 registry）
    /// 开关是否开启（仅在已解锁时可生效）
    pub developer_mode: Option<bool>,

    // TLS（trust_mode 进 AppConfig；ignore_tls 走注册表，仅在开发者模式可开启）
    /// TLS 信任源模式：builtin / system / custom / system+custom / builtin+custom / all
    #[serde(rename = "tlsTrustMode")]
    pub tls_trust_mode: Option<String>,
    /// 是否忽略 TLS 证书校验（开发者模式注册表键，仅在 developer_mode 开启时可生效）
    #[serde(rename = "ignoreTls")]
    pub ignore_tls: Option<bool>,

    // 正版购买提示（系统存储：Windows 注册表 / 其他系统全局共用文件，不进 AppConfig）
    /// 游戏启动成功次数（正版购买提示计数）
    pub launch_count: Option<u32>,
    /// 是否永久忽略正版购买提示
    pub hint_buy: Option<bool>,
    /// 是否永久忽略"去 GitHub 点 Star"提示
    pub hint_star: Option<bool>,
    // 用户协议（系统存储：全局首次启动门禁，不进 AppConfig）
    /// 是否已同意《用户协议》
    pub user_agreed: Option<bool>,
    /// 已同意的《用户协议》版本号
    pub user_agreed_version: Option<u32>,
}
