//! Frp 模块统一分发逻辑（frp_manager 的命令层实现）
//! action 注册按类别拆分到子模块，本文件保留参数结构体与 DISPATCHER 入口。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use super::{auth_actions, provider_actions, public_server_actions, tunnel_actions};
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// 参数结构体

/// ensure_frpc 参数（provider_id 可选，默认系统默认厂商；force=true 强制重新下载）
#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnsureFrpcParams {
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub force: bool,
}

/// 安装厂商参数（source_dir 可为文件夹路径或 ZIP 路径）
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProviderParams {
    pub source_dir: String,
}

/// 从 URL 安装厂商参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProviderFromUrlParams {
    pub url: String,
}

/// 厂商 ID 参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIdParams {
    pub provider_id: String,
}

/// 读取日志参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadLogParams {
    pub tunnel_id: String,
    #[serde(default)]
    pub max_lines: Option<usize>,
}

/// 保存 API Key 参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveApiKeyParams {
    pub provider_id: String,
    pub api_key: String,
}

/// 执行厂商认证适配器脚本参数（对应 §7.5 沙箱）
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunAuthAdapterParams {
    pub provider_id: String,
    /// 要执行的命令（必须在厂商 allowedCommands 白名单内）
    pub command: String,
    /// 命令参数
    #[serde(default)]
    pub args: Vec<String>,
}

/// 拖拽包类型检测参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectPackageParams {
    /// 文件/文件夹路径（ZIP 或目录）
    pub path: String,
}

// DISPATCHER 注册（按类别委托到子模块）

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    provider_actions::register(&mut d);
    tunnel_actions::register(&mut d);
    auth_actions::register(&mut d);
    public_server_actions::register(&mut d);
    d
});

/// Frp 管理 action 分发入口
pub async fn dispatch(
    _state: AppState,
    _app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(_state, _app, req).await
}
