//! 配置数据类型：补丁、快照、条目
//!
//! - `ConfigPatch`：`apply_config` 入参，所有字段 `Option<T>`，仅传需要改的字段
//! - `ConfigSnapshot`：`get_config` 返回的全量配置快照
//! - `ConfigEntry`：扁平化 key-value 对，前后端 IPC 格式对称

use serde::{Deserialize, Serialize};

/// 配置补丁：所有字段可选，仅传需要更新的字段
///
/// 字段命名采用 camelCase 序列化（前端约定），与 `AppConfig` 的 snake_case
/// 字段一一对应（通过 `#[serde(rename_all = "camelCase")]` 映射）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPatch {
    // ===== 代理 =====
    pub proxy_mode: Option<String>,
    pub proxy_type: Option<String>,
    pub proxy_url: Option<String>,

    // ===== 下载 =====
    /// "official" / "mirror" / "smart"
    pub download_source: Option<String>,
    pub meta_source: Option<String>,
    pub max_download_speed: Option<u64>,
    pub max_download_threads: Option<u32>,
    pub chunk_count: Option<u32>,
    /// 双层 Option：外层 Some 表示"要更新此字段"，内层 None 表示"清空"
    pub mirror_url: Option<Option<String>>,

    // ===== 内存 =====
    pub memory_mode: Option<String>,
    pub min_memory: Option<u32>,
    pub max_memory: Option<u32>,

    // ===== 启动器 =====
    pub game_dir: Option<String>,
    pub isolation_mode: Option<u32>,
    pub log_level: Option<u32>,
    pub selected_version: Option<Option<String>>,
    /// 游戏默认界面语言：写入 options.txt 的 lang 字段
    /// - "auto"：跟随启动器语言（旧配置兼容）
    /// - "zh_cn" / "en_us" / "ja_jp" / "ko_kr" 等 MC 标准语言代码
    /// - "none"：不设置
    pub game_language: Option<String>,
    /// 主题主色 HEX（如 "#165dff"），前端注入 CSS 变量驱动 primary-* 色阶
    pub primary_color: Option<String>,

    // ===== 社区资源（INI 明文，进 AppConfig）=====
    pub community_source: Option<u8>,
    pub community_filename_format: Option<u8>,
    pub community_mod_local_name_style: Option<u8>,
    pub community_ignore_quilt: Option<bool>,

    // ===== CurseForge（加密存储，不进 AppConfig，内部分流到 secure_storage）=====
    pub curseforge_enabled: Option<bool>,
    pub curseforge_api_key: Option<String>,

    // ===== 启动高级选项 =====
    pub launch_disable_jlw: Option<bool>,
    pub launch_disable_lua: Option<bool>,
    pub launch_use_dedicated_gpu: Option<bool>,

    // ===== 开发者模式（注册表存储，不进 AppConfig，内部分流到 registry）=====
    /// 开关是否开启（仅在已解锁时可生效）
    pub developer_mode: Option<bool>,
}

/// 配置快照：返回所有配置字段的当前值
///
/// 用于前端一次性读取全部配置，取代此前分散的 14 个 get_* 命令。
/// CurseForge 的 api_key 从 secure_storage 缓存读取（已解密），
/// 若首次未解密则返回空字符串（懒加载，避免触发杀软误报）。
/// 开发者模式从注册表读取（DeveloperUnlocked / DeveloperMode）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSnapshot {
    // 代理
    pub proxy_mode: String,
    pub proxy_type: String,
    pub proxy_url: String,
    // 下载
    pub mirror_url: Option<String>,
    pub download_source: String,
    pub meta_source: String,
    pub max_download_speed: u64,
    pub max_download_threads: u32,
    pub chunk_count: u32,
    // 内存
    pub memory_mode: String,
    pub min_memory: u32,
    pub max_memory: u32,
    // 启动器
    pub game_dir: String,
    pub isolation_mode: u32,
    pub log_level: u32,
    pub selected_version: Option<String>,
    /// 游戏默认界面语言
    pub game_language: String,
    /// 主题主色 HEX（如 "#165dff"）
    pub primary_color: String,
    // 社区资源（INI 明文）
    pub community_source: u8,
    pub community_filename_format: u8,
    pub community_mod_local_name_style: u8,
    pub community_ignore_quilt: bool,
    // CurseForge（从 secure_storage 缓存读，已解密）
    pub curseforge_enabled: bool,
    pub curseforge_api_key: String,
    // 启动高级选项
    pub launch_disable_jlw: bool,
    pub launch_disable_lua: bool,
    pub launch_use_dedicated_gpu: bool,
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
        proxy_mode: config.proxy_mode.clone(),
        proxy_type: config.proxy_type.clone(),
        proxy_url: config.proxy_url.clone(),
        mirror_url: config.mirror_url.clone(),
        download_source: config.download_source.clone(),
        meta_source: config.meta_source.clone(),
        max_download_speed: config.max_download_speed,
        max_download_threads: config.max_download_threads,
        chunk_count: config.chunk_count,
        memory_mode: config.memory_mode.clone(),
        min_memory: config.min_memory,
        max_memory: config.max_memory,
        game_dir: config.game_dir.clone(),
        isolation_mode: config.isolation_mode,
        log_level: config.log_level,
        selected_version: config.selected_version.clone(),
        game_language: config.game_language.clone(),
        primary_color: config.primary_color.clone(),
        community_source: config.community_source,
        community_filename_format: config.community_filename_format,
        community_mod_local_name_style: config.community_mod_local_name_style,
        community_ignore_quilt: config.community_ignore_quilt,
        curseforge_enabled: cf_enabled,
        curseforge_api_key: cf_api_key.unwrap_or_default(),
        launch_disable_jlw: config.launch_disable_jlw,
        launch_disable_lua: config.launch_disable_lua,
        launch_use_dedicated_gpu: config.launch_use_dedicated_gpu,
        developer_unlocked: dev_unlocked,
        developer_mode: dev_mode,
    }
}
