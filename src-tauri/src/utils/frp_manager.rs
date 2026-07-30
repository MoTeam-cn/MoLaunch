//! Frp 模块统一分发逻辑（frp_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，覆盖：
//! - 厂商管理：list_providers / enable_provider / disable_provider（provider）；
//!   ensure_frpc（binary）；install_provider_from_dir / install_provider_from_zip /
//!   uninstall_provider（install）
//! - 隧道管理：list_tunnels / create_tunnel / delete_tunnel
//! - frpc 进程：start_tunnel / stop_tunnel / get_tunnel_status
//! - 日志管理：list_log_files / read_log_file
//!
//! start_tunnel 需要 `AppHandle` 用于推送 frpc-log / frp-tunnel-status event。
//! ensure_frpc 可选接收 provider_id 参数（默认系统默认厂商）。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::commands::frp;
use crate::commands::frp::tunnel::{CreateTunnelParams, TunnelIdParams};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// 参数结构体
// ============================================================

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

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // ============================================================
    // 厂商管理
    // ============================================================

    d.register("list_providers", handler!(_state, _app, _params, {
        let r = frp::provider::list_providers().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("ensure_frpc", handler!(_state, _app, params, {
        // 兼容空 params（{} 或 null）：unwrap_or_default 返回 provider_id=None
        let p: EnsureFrpcParams = serde_json::from_value(params).unwrap_or_default();
        let r = frp::binary::ensure_frpc(p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("install_provider_from_dir", handler!(_state, _app, params, {
        let p: InstallProviderParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::install::install_provider_from_dir(p.source_dir).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("install_provider_from_zip", handler!(_state, _app, params, {
        let p: InstallProviderParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::install::install_provider_from_zip(p.source_dir).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("uninstall_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::install::uninstall_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("enable_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::provider::enable_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("disable_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::provider::disable_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // ============================================================
    // 隧道管理
    // ============================================================

    d.register("list_tunnels", handler!(_state, _app, _params, {
        let r = frp::process::list_tunnels_with_status().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("create_tunnel", handler!(_state, _app, params, {
        let p: CreateTunnelParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::sandbox::validate_tunnel(&p)?;
        let r = frp::tunnel::create_tunnel(p).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("delete_tunnel", handler!(_state, _app, params, {
        let p: TunnelIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        // 先停止运行中的进程（忽略错误，可能未在运行）
        let _ = frp::process::stop_tunnel(p.id.clone()).await;
        frp::tunnel::delete_tunnel(p.id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // ============================================================
    // frpc 进程管理
    // ============================================================

    d.register("start_tunnel", handler!(_state, app, params, {
        let p: TunnelIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::process::start_tunnel(p.id, app.clone()).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("stop_tunnel", handler!(_state, _app, params, {
        let p: TunnelIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::process::stop_tunnel(p.id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("get_tunnel_status", handler!(_state, _app, params, {
        let p: TunnelIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::process::get_tunnel_status(p.id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // ============================================================
    // 日志管理
    // ============================================================

    d.register("list_log_files", handler!(_state, _app, _params, {
        let r = frp::process::list_log_files().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("read_log_file", handler!(_state, _app, params, {
        let p: ReadLogParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::process::read_log_file(p.tunnel_id, p.max_lines).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

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
