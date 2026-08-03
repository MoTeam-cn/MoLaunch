//! 插件系统共享类型（manifest 结构 + 权限配置）

use serde::{Deserialize, Serialize};

/// 子进程权限配置（manifest.json 的 processPermissions 字段）
///
/// 仅当 permissions 包含 "spawnProcess" 时此字段才生效。
/// 用于约束插件可执行的命令，防止任意命令执行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPermissions {
    /// 允许执行的命令白名单（不含参数，如 ["java", "node", "python"]）
    ///
    /// 命令名会被 canonicalize 后与白名单匹配，支持绝对路径或 PATH 查找结果。
    /// 不在白名单内的命令会被拒绝。
    #[serde(default)]
    pub allowed_commands: Vec<String>,
    /// 单次执行超时（毫秒），默认 30000（30 秒），最大 300000（5 分钟）
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// 最大并发进程数，默认 1，最大 5
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
}

fn default_timeout_ms() -> u64 {
    30000
}

fn default_max_concurrent() -> u32 {
    1
}

/// 窗口权限配置（manifest.json 的 windowPermissions 字段）
///
/// 仅当 permissions 包含 "createWindow" 时此字段才生效。
/// 用于约束插件可创建的子窗口行为，防止任意 URL 弹窗。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPermissions {
    /// 允许打开的 URL 域名白名单（如 ["example.com", "*.github.io"]）
    ///
    /// 支持通配符前缀 `*.` 匹配子域名。
    /// 不在白名单内的 URL 会被拒绝。
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    /// 窗口默认宽度（像素），默认 800
    #[serde(default = "default_window_width")]
    pub width: f64,
    /// 窗口默认高度（像素），默认 600
    #[serde(default = "default_window_height")]
    pub height: f64,
    /// 是否允许窗口大小可调整，默认 true
    #[serde(default = "default_window_resizable")]
    pub resizable: bool,
}

fn default_window_width() -> f64 {
    800.0
}

fn default_window_height() -> f64 {
    600.0
}

fn default_window_resizable() -> bool {
    true
}

/// 外部插件清单（对应 manifest.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPluginManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    /// HTML 入口文件相对路径（相对插件目录，如 "index.html"）
    pub entry: String,
    /// 权限白名单（SDK 方法名数组）
    #[serde(default)]
    pub permissions: Vec<String>,
    /// 子进程权限配置（仅当 permissions 包含 "spawnProcess" 时生效）
    #[serde(default)]
    pub process_permissions: Option<ProcessPermissions>,
    /// 窗口权限配置（仅当 permissions 包含 "createWindow" 时生效）
    #[serde(default)]
    pub window_permissions: Option<WindowPermissions>,
}

/// 已扫描到的外部插件（含目录路径）
#[derive(Debug, Clone, Serialize)]
pub struct ExternalPluginEntry {
    #[serde(flatten)]
    pub manifest: ExternalPluginManifest,
    /// 插件目录绝对路径
    pub dir: String,
}