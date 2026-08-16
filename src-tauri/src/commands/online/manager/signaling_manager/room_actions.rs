//! 房间生命周期 action 注册：创建/查询/加入/关闭/心跳（Scaffolding 收敛版）。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::signaling::CreateRoomRequest;
use crate::utils::dispatcher::Dispatcher;

use super::{CreateRoomParams, JoinRoomParams, RoomCodeParams};

/// 注册房间生命周期相关 action
pub fn register(d: &mut Dispatcher) {
    register_create_room(d);
    register_get_room(d);
    register_join_room(d);
    register_close_room(d);
    register_heartbeat_room(d);
}

fn register_create_room(d: &mut Dispatcher) {
    d.register("room_create", handler!(state, _app, params, {
        let p: CreateRoomParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = super::load_creds(&state).await?;
        let client = super::make_client(&state).await;
        log_info!(
            "[Online] room_create: code={}, is_public={}, has_password={}, mc_port={}, modpack={}",
            p.room_code, p.is_public, !p.password.is_empty(), p.host_mc_port,
            p.modpack.as_ref().map(|m| format!("{}({}:{})", m.source, m.project_id, m.file_id)).unwrap_or_else(|| "none".to_string())
        );
        let req = CreateRoomRequest {
            room_code: p.room_code,
            remark: p.remark,
            is_public: p.is_public,
            password: p.password,
            host_mc_version: p.host_mc_version,
            host_mc_port: p.host_mc_port,
            host_loader: p.host_loader,
            host_loader_version: p.host_loader_version,
            modpack: p.modpack,
        };
        let result = client.signaling_create_room(&creds, &req).await
            .map_err(|e| {
                log_error!("[Online] room_create 失败: {}", e);
                e.to_string()
            })?;
        if let Some(ref data) = result.data {
            log_info!("[Online] 房间登记成功: room_code={}", data.room_code);
        }
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_get_room(d: &mut Dispatcher) {
    d.register(
        "room_get",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] room_get: code={}", p.room_code);
            let result = client
                .signaling_get_room(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_get 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_join_room(d: &mut Dispatcher) {
    d.register(
        "room_join",
        handler!(state, _app, params, {
            let p: JoinRoomParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!(
                "[Online] room_join: code={}, has_password={}",
                p.room_code,
                !p.password.is_empty()
            );
            let result = client
                .signaling_join_room(&creds, &p.room_code, &p.password)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_join 失败: {}", e);
                    e.to_string()
                })?;
            if let Some(ref data) = result.data {
                log_info!("[Online] 加入房间成功: room_code={}", data.room_code);
            }
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_close_room(d: &mut Dispatcher) {
    d.register(
        "room_close",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!("[Online] room_close: code={}", p.room_code);
            let result = client
                .signaling_close_room(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_close 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_heartbeat_room(d: &mut Dispatcher) {
    d.register(
        "room_heartbeat",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] room_heartbeat: code={}", p.room_code);
            let result = client
                .signaling_heartbeat_room(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_heartbeat 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}
