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
    /// 受限 CurseForge 文件是否自动推导 CDN 下载地址（开发者选项，默认关闭）
    ///
    /// CurseForge 作者可以禁止第三方分发，此时官方 API 不返回下载地址。开启后按
    /// fileId 与文件名推导 `edge.forgecdn.net` 地址直接下载；关闭时这些文件不做
    /// 推导，由调用方给出手动下载提示。推导依赖 CurseForge 未公开的 CDN 路径布局，
    /// 因此默认关闭，只在「开发者 → 实验性功能」显式开启后生效。
    #[serde(default)]
    pub cf_derive_url: bool,
    /// 是否启用 DoH 解析（开发者选项，默认关闭）
    ///
    /// 开启后下载域名优先走 DNS-JSON 接口解析（腾讯 → 阿里 → Cloudflare 依次回落），
    /// 全部失败仍回落系统解析。默认关闭：国内 DoH 端点可用性不稳定，且域名会发给
    /// 第三方解析商，属于隐私边界，不替用户默认决定。
    #[serde(default)]
    pub doh: bool,
    /// 是否对公开 CDN 做边缘节点优选（实验，默认关闭）
    ///
    /// 开启后对 `cdn.modrinth.com` 这类多边缘域名解析出的多个 IP 各拉一小段
    /// 真实文件测吞吐，挑最快的钉住使用（证书与 Host 仍是原域名）。会消耗少量
    /// 探测流量（每小时上限 64 MiB），且只在不走代理、未关闭证书校验时生效。
    #[serde(default)]
    pub edge_select: bool,
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
    /// 是否启用联机大厅（用户显式开启并同意与云端交互后为 true）
    ///
    /// 为 false 时启动流程不发起任何云端请求（不静默注册、不续期 token），
    /// 联机大厅/房间等云端功能整体不可用；FRP 使用独立凭证体系，不受此开关影响。
    #[serde(default)]
    pub lobby_enabled: bool,
    #[serde(default)]
    pub custom_turn_servers: Vec<IceServerEntry>,
    /// easytier-core 可执行文件路径（绝对路径；留空使用内置嵌入式资源，释放到 AppData/.Molaunch/easytier/）
    #[serde(default = "crate::state::config::defaults::default_easytier_core_path")]
    pub easytier_core_path: String,
    /// 虚拟网络内节点标识（预留字段，用于房客侧 easytier hostname）
    #[serde(default)]
    pub network_identity: String,
    /// 公共 easytier 中继节点列表（`--peers` 参数，格式 `tcp://host:port` / `udp://host:port`；用于公网组网穿越 NAT）
    #[serde(default)]
    pub easytier_public_peers: Vec<String>,
    /// 用户自定义 GitHub 镜像源（easytier 内核等外部二进制下载竞速选源用，支持 full/path 模式）
    #[serde(default)]
    pub github_proxies: Vec<crate::utils::github_download::GithubProxy>,
}

impl Default for OnlineConfig {
    fn default() -> Self {
        Self {
            api_server_url: "https://api.molaunch.moiu.cn".to_string(),
            lobby_enabled: false,
            custom_turn_servers: Vec::new(),
            easytier_core_path: crate::state::config::defaults::default_easytier_core_path(),
            network_identity: String::new(),
            easytier_public_peers: Vec::new(),
            github_proxies: Vec::new(),
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
    /// 启动器界面 GPU 硬件加速（默认开启；关闭后 WebView2 走软件渲染，降低内存占用，需重启生效）
    #[serde(default = "crate::state::config::defaults::default_use_gpu_acceleration")]
    pub use_gpu_acceleration: bool,
    /// 关闭到托盘时挂起 WebView2 释放渲染资源（默认关闭；恢复托盘窗口时自动恢复并保留界面状态）
    #[serde(default = "crate::state::config::defaults::default_release_memory_on_tray")]
    pub release_memory_on_tray: bool,
    pub proxy: ProxyConfig,
    pub download: DownloadConfig,
    pub memory: MemoryConfig,
    pub community: CommunityConfig,
    pub launch_advanced: LaunchAdvancedConfig,
    /// 默认日志分享服务："mclogs"（mclo.gs）/ "logshare"（logshare.cn）
    #[serde(default = "crate::state::config::defaults::default_log_share_provider")]
    pub log_share_provider: String,
    /// 更新源："api"（MoLaunch 云端公开端点，默认）/ "github"（GitHub Releases，备用）
    ///
    /// 两个源均无需设备注册与 JWT，检查更新不受联机大厅开关影响。
    pub online: OnlineConfig,
    pub tls: TlsConfig,
}

/// Minecraft 文件夹项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McFolder {
    pub name: String,
    pub path: String,
}
