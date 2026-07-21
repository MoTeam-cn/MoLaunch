//! 应用配置（AppConfig + McFolder + 路径解析）

use serde::{Deserialize, Serialize};

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub game_dir: String,
    /// Minecraft 文件夹列表（含默认和用户添加的）
    pub mc_folders: Vec<McFolder>,
    pub max_download_threads: u32,
    pub chunk_count: u32,
    pub isolation_mode: u32,
    pub log_level: u32,
    pub min_memory: u32,
    pub max_memory: u32,
    pub memory_mode: String, // "auto" | "custom"
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
    pub mirror_url: Option<String>,
    pub mirror_url_meta: Option<String>,
    pub mirror_url_download: Option<String>,
    pub mirror_mode: u32,
    pub max_download_speed: u64,
    pub download_source: String, // "mirror" | "official" | "smart" — 文件下载源
    pub meta_source: String,     // "mirror" | "official" | "smart" — 版本列表源
    pub proxy_mode: String,      // "none" | "system" | "custom"
    pub proxy_type: String,      // "http" | "https" | "socks5"
    pub proxy_url: String,       // 自定义代理地址，如 "127.0.0.1:7890"
    /// 上次选中的游戏版本（持久化，启动器重启后恢复）
    pub selected_version: Option<String>,
    // ===== 社区资源配置 =====
    /// 社区资源来源策略：0=尽量镜像 / 1=缓慢时换镜像 / 2=尽量官方（默认 2）
    pub community_source: u8,
    /// 下载文件名格式：0=【译名】原名 / 1=[译名] 原名 / 2=译名-原名 / 3=原名-译名 / 4=仅原名（默认 1）
    pub community_filename_format: u8,
    /// Mod 管理页显示样式：0=标题译名/详情文件名 / 1=标题文件名/详情译名（默认 0）
    pub community_mod_local_name_style: u8,
    /// 在显示 Mod 加载器时忽略 Quilt（默认 true）
    pub community_ignore_quilt: bool,
    // ===== 启动高级选项 =====
    /// 禁用 Java Launch Wrapper（默认 false）
    /// JLW 用于修复 Java 18- 在中文路径下可能无法正常启动的问题
    pub launch_disable_jlw: bool,
    /// 禁用 LWJGL Unsafe Agent（默认 false）
    /// LUA 用于修复 LWJGL 3.4.1 的一个性能问题
    pub launch_disable_lua: bool,
    /// 使用高性能显卡（默认 false）
    /// 自动在 Windows 设置中将启动器和 Java 改为使用高性能显卡
    pub launch_use_dedicated_gpu: bool,
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
            max_download_threads: 8,
            chunk_count: 4,
            isolation_mode: 4,
            log_level: 3,
            min_memory: 0,
            max_memory: 0,
            memory_mode: "auto".to_string(),
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            game_language: "zh_cn".to_string(),
            primary_color: "#165dff".to_string(),
            mirror_url: None,
            mirror_url_meta: None,
            mirror_url_download: None,
            mirror_mode: 0,
            max_download_speed: 0,
            download_source: "smart".to_string(),
            meta_source: "smart".to_string(),
            proxy_mode: "none".to_string(),
            proxy_type: "http".to_string(),
            proxy_url: String::new(),
            selected_version: None,
            community_source: 2,
            community_filename_format: 1,
            community_mod_local_name_style: 0,
            community_ignore_quilt: true,
            launch_disable_jlw: false,
            launch_disable_lua: false,
            launch_use_dedicated_gpu: false,
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
