//! 隧道管理 action 注册：隧道 CRUD、frpc 进程启停/状态、日志读取。

use crate::commands::frp;
use crate::commands::frp::tunnel::{CreateTunnelParams, TunnelIdParams, UpdateTunnelParams};
use crate::handler;
use crate::utils::dispatcher::Dispatcher;

use super::ReadLogParams;

/// 注册隧道管理相关 action
pub fn register(d: &mut Dispatcher) {
    // 隧道 CRUD
    d.register(
        "list_tunnels",
        handler!(_state, _app, _params, {
            let r = frp::process::list_tunnels_with_status().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "create_tunnel",
        handler!(_state, _app, params, {
            let p: CreateTunnelParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::sandbox::validate_tunnel(&p)?;
            let r = frp::tunnel::create_tunnel(p).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "delete_tunnel",
        handler!(_state, _app, params, {
            let p: TunnelIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            // 先停止运行中的进程（忽略错误，可能未在运行）
            let _ = frp::process::stop_tunnel(p.id.clone()).await;
            frp::tunnel::delete_tunnel(p.id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "update_tunnel",
        handler!(_state, _app, params, {
            let p: UpdateTunnelParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::sandbox::validate_tunnel_update(&p)?;
            let r = frp::tunnel::update_tunnel(p).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    // frpc 进程管理
    d.register(
        "start_tunnel",
        handler!(state, app, params, {
            let p: TunnelIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::process::start_tunnel(&state, p.id, app.clone()).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "stop_tunnel",
        handler!(_state, _app, params, {
            let p: TunnelIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::process::stop_tunnel(p.id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_tunnel_status",
        handler!(_state, _app, params, {
            let p: TunnelIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::process::get_tunnel_status(p.id).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    // 日志管理
    d.register(
        "list_log_files",
        handler!(_state, _app, _params, {
            let r = frp::process::list_log_files().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "read_log_file",
        handler!(_state, _app, params, {
            let p: ReadLogParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::process::read_log_file(p.tunnel_id, p.max_lines).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
}
