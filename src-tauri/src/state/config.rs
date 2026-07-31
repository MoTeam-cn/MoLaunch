//! 应用配置（AppConfig + McFolder + 路径解析）

use crate::minecraft::online::signaling::IceServerEntry;
use serde::{Deserialize, Serialize};

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyConfig {
    pub mode: String, // "none" | "system" | "custom"（原 proxy_mode）
    pub kind: String, // "http" | "https" | "socks5"（原 proxy_type，避开 Rust 关键字 type）
    pub url: String,  // 自定义代理地址，如 "127.0.0.1:7890"（原 proxy_url）
    /// IP 协议版本偏好
    /// - `"v4"`: 强制 IPv4（local_address = 0.0.0.0）
    /// - `"auto"`: 自动选择（测试 v4/v6 连通性，选稳定的那个）
    /// - `"any"`: 随意解析（不设置 local_address，跟随 DNS 服务器）
    #[serde(default)]
    pub ip_version: String,
}

/// 下载配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadConfig {
    pub source: String,                      // "mirror" | "official" | "smart" — 文件下载源（原 download_source）
    pub meta_source: String,                 // "mirror" | "official" | "smart" — 版本列表源（原 meta_source）
    pub max_speed: u64,                      // 原 max_download_speed
    pub max_threads: u32,                    // 原 max_download_threads
    pub chunk_count: u32,                    // 原 chunk_count
    pub mirror_url: Option<String>,          // 原 mirror_url
    pub mirror_url_meta: Option<String>,     // 原 mirror_url_meta
    pub mirror_url_download: Option<String>, // 原 mirror_url_download
    pub mirror_mode: u32,                    // 原 mirror_mode
    /// 是否将 `cdn.modrinth.com` 替换为 `cdn-raw.modrinth.com`（绕过中国大陆 cdn-alt 跳转）
    ///
    /// 默认 false（关闭）。仅在开发者模式解锁后可在「设置 → 开发者模式」中开启。
    /// 开启后 `sources::rewrite_mr_cdn` 生效，所有 Modrinth CDN 下载 URL 入口处
    /// 先做域名替换，再按 source 策略走镜像/官方。
    #[serde(default)]
    pub modrinth_cdn_raw_enabled: bool,
}

/// 内存配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    pub mode: String, // "auto" | "custom"（原 memory_mode）
    pub min: u32,     // 原 min_memory
    pub max: u32,     // 原 max_memory
}

/// 社区资源配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommunityConfig {
    /// 社区资源来源策略：0=尽量镜像 / 1=缓慢时换镜像 / 2=尽量官方（默认 2）
    pub source: u8, // 原 community_source
    /// 下载文件名格式：0=【译名】原名 / 1=[译名] 原名 / 2=译名-原名 / 3=原名-译名 / 4=仅原名（默认 1）
    pub filename_format: u8, // 原 community_filename_format
    /// Mod 管理页显示样式：0=标题译名/详情文件名 / 1=标题文件名/详情译名（默认 0）
    pub mod_local_name_style: u8, // 原 community_mod_local_name_style
    /// 在显示 Mod 加载器时忽略 Quilt（默认 true）
    pub ignore_quilt: bool, // 原 community_ignore_quilt
}

/// 启动高级选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchAdvancedConfig {
    /// 禁用 Java Launch Wrapper（默认 false）
    /// JLW 用于修复 Java 18- 在中文路径下可能无法正常启动的问题
    pub disable_jlw: bool, // 原 launch_disable_jlw
    /// 禁用 LWJGL Unsafe Agent（默认 false）
    /// LUA 用于修复 LWJGL 3.4.1 的一个性能问题
    pub disable_lua: bool, // 原 launch_disable_lua
    /// 使用高性能显卡（默认 false）
    /// 自动在 Windows 设置中将启动器和 Java 改为使用高性能显卡
    pub use_dedicated_gpu: bool, // 原 launch_use_dedicated_gpu
}

/// 联机功能配置
///
/// 持久化「api_server_url」+「custom_turn_servers」两项：
/// - `api_server_url`：联机服务端地址（默认生产地址）
/// - `custom_turn_servers`：用户自定义 TURN 服务器列表（阶段三子任务 7 新增）
///
/// 设备密钥与 JWT 不在此处，由 `minecraft::online::storage` 独立持久化
/// （走 SDK DES 加密 + 单独文件，与 auth_storage 一致的位置）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineConfig {
    /// 联机 api-server 地址（默认 `https://api.molaunch.moiu.cn`）
    ///
    /// 用户可在「设置 → 联机」页修改。前端通过 `applyConfig({ onlineApiServerUrl })` 更新，
    /// 后端 `online_manager` 读取此值构造 `OnlineClient`。
    pub api_server_url: String,
    /// 用户自定义 TURN 服务器列表（阶段三子任务 7 新增）
    ///
    /// 默认空数组。用户可在「设置 → 联机 → ICE 服务器配置」页增删条目，前端通过
    /// `applyConfig({ onlineCustomTurnServers })` 持久化。房主创建房间时与系统 STUN/TURN
    /// 合并后上报后端，作为房间内 P2P 兜底中转备用方案。
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
///
/// `trust_mode` 控制信任源组合（`http::build_client` 据此加载根证书）：
/// `builtin`(webpki-roots) / `system`(OS 根证书) / `custom`(certs 目录 PEM)，
/// 支持 `system+custom`、`builtin+custom` 组合与 `all` 全加载。
/// `IgnoreTls`（开发者模式注册表键）开启时跳过所有证书校验，此字段不生效。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// 信任源模式：system / builtin / custom / system+custom / builtin+custom / all
    #[serde(default = "default_trust_mode")]
    pub trust_mode: String,
}

fn default_trust_mode() -> String {
    "builtin".to_string()
}

impl Default for TlsConfig {
    fn default() -> Self {
        TlsConfig {
            trust_mode: default_trust_mode(),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // 通用（保持平铺）
    pub game_dir: String,
    /// Minecraft 文件夹列表（含默认和用户添加的）
    pub mc_folders: Vec<McFolder>,
    pub isolation_mode: u32,
    pub log_level: u32,
    pub theme: String,
    pub language: String,
    /// 游戏默认界面语言（写入 options.txt 的 lang 字段）
    /// - "zh_cn" / "en_us" / "ja_jp" / "ko_kr" 等 MC 标准语言代码（默认 "zh_cn"）
    /// - "none"：不设置，保留玩家游戏内手动选择
    /// - "auto"：旧配置兼容值，后端 resolve_game_language 会按启动器语言映射处理
    pub game_language: String,
    /// 主题主色 HEX（如 "#165dff"），前端读取后通过 applyPrimaryColor() 注入 CSS 变量
    /// 驱动 Tailwind primary-* 色阶与 main.css 中所有 var(--color-primary-*)
    pub primary_color: String,
    /// 上次选中的游戏版本（持久化，启动器重启后恢复）
    pub selected_version: Option<String>,
    // 外部下载工具
    /// 外部下载工具的自定义保存目录（None 或空则用默认 .Molaunch/Download/）
    pub external_download_dir: Option<String>,

    // 分组
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
    /// 显示名称
    pub name: String,
    /// 文件夹路径（相对或绝对）
    pub path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_game_dir = get_default_game_dir();
        Self {
            game_dir: default_game_dir.clone(),
            mc_folders: vec![McFolder {
                name: "默认".to_string(),
                path: default_game_dir,
            }],
            isolation_mode: 4,
            log_level: 3,
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            game_language: "zh_cn".to_string(),
            primary_color: "#165dff".to_string(),
            selected_version: None,
            external_download_dir: None,
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
            online: OnlineConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

/// 获取默认游戏目录：启动器同级目录下的 .minecraft
pub(crate) fn get_default_game_dir() -> String {
    // 优先使用可执行文件所在目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let minecraft_dir = exe_dir.join(".minecraft");
            return minecraft_dir.to_string_lossy().to_string();
        }
    }
    // 兜底：当前工作目录
    if let Ok(cwd) = std::env::current_dir() {
        let minecraft_dir = cwd.join(".minecraft");
        return minecraft_dir.to_string_lossy().to_string();
    }
    ".minecraft".to_string()
}

/// 解析游戏目录：如果是相对路径，则相对于可执行文件目录
pub fn resolve_game_dir(game_dir: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(game_dir);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    // 相对路径：优先相对于可执行文件目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(game_dir);
        }
    }
    // 兜底：当前工作目录
    std::env::current_dir().unwrap_or_default().join(game_dir)
}
