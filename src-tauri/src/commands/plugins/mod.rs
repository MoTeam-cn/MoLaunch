//! 插件系统命令模块（编排层）
//!
//! 外部插件存放于 `<base_dir>/plugins/<plugin_id>/`，每个插件目录包含
//! manifest.json 和入口 HTML。子模块按职责拆分：install / sandbox / spawn /
//! window / layout / export / personalization。
//! 所有子模块函数由 `utils::plugins_manager::dispatch` 统一反序列化参数后调用。

pub mod export;
pub mod install;
pub mod layout;
pub mod personalization;
pub mod sandbox;
pub mod spawn;
pub mod window;

use crate::error_util::log_err;
use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, State};


/// 统一插件系统 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::plugins_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn plugins_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::plugins_manager::dispatch(state, app, req).await
}

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

/// 获取外部插件根目录（`<base_dir>/plugins/`）
fn plugins_root() -> PathBuf {
    crate::storage::Storage::instance().base_dir().join("plugins")
}

/// 校验插件 ID 合法性（kebab-case，仅允许小写字母、数字、连字符）
fn is_valid_plugin_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

/// 读取插件 manifest.json
///
/// 校验 plugin_id 合法性 + manifest.id 与目录名一致。
fn read_plugin_manifest(plugin_id: &str) -> Result<ExternalPluginManifest, String> {
    if !is_valid_plugin_id(plugin_id) {
        return Err(format!("Invalid plugin id: {}", plugin_id));
    }

    let plugin_dir = plugins_root().join(plugin_id);
    let manifest_path = plugin_dir.join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!("Plugin manifest not found: {}", manifest_path.display()));
    }

    let manifest_str = std::fs::read_to_string(&manifest_path).map_err(log_err("Failed to read plugin manifest"))?;
    let manifest: ExternalPluginManifest =
        serde_json::from_str(&manifest_str).map_err(|e| format!("Invalid manifest.json: {}", e))?;

    // 校验 manifest.id 与目录名一致
    if manifest.id != plugin_id {
        return Err(format!(
            "manifest.id ({}) 与目录名 ({}) 不一致",
            manifest.id, plugin_id
        ));
    }

    Ok(manifest)
}
