//! Frp 模块统一分发逻辑（frp_manager 的命令层实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，覆盖厂商/隧道/进程/日志/
//! 认证/厂商 API 引擎/认证适配器沙箱/公共 frps 服务器等 action。
//! action 注册按类别拆分到子模块，主文件保留参数结构体与 DISPATCHER 入口。

mod auth_actions;
mod provider_actions;
mod public_server_actions;
mod tunnel_actions;

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// 参数结构体

/// ensure_frpc 参数（provider_id 可选，默认系统默认厂商）
#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnsureFrpcParams {
    #[serde(default)]
    pub provider_id: Option<String>,
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

/// 分配公共服务器端口参数（对应 apiServer `AllocateRequest`）
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocatePublicServerParams {
    pub server_id: String,
    pub tunnel_type: String,
}

/// 释放/续期分配参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationIdParams {
    pub allocation_id: String,
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
