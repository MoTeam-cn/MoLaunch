//! 房间生命周期 action 注册：STUN/创建/查询/关闭/加入/退出/保活/TURN 拉取。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::signaling::CreateRoomRequest;
use crate::utils::dispatcher::Dispatcher;

use super::{CreateRoomParams, JoinRoomParams, RoomCodeParams};

/// 注册房间生命周期相关 action
pub fn register(d: &mut Dispatcher) {
    register_get_stun(d);
    register_create_room(d);
    register_get_room(d);
    register_close_room(d);
    register_join_room(d);
    register_keepalive(d);
    register_get_turn_servers(d);
    register_leave_room(d);
}

fn register_get_stun(d: &mut Dispatcher) {
    d.register(
        "room_get_stun",
        handler!(state, _app, _params, {
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] room_get_stun");
            let result = client.signaling_get_stun(&creds).await.map_err(|e| {
                log_error!("[Online] room_get_stun 失败: {}", e);
                e.to_string()
            })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_create_room(d: &mut Dispatcher) {
    d.register("room_create", handler!(state, _app, params, {
        let p: CreateRoomParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = super::load_creds(&state).await?;
        let client = super::make_client(&state).await;
        log_info!(
            "[Online] room_create: max_players={}, mc_version={}, mc_port={}, loader={:?}, loader_version={:?}, room_type={}, lobby_id={:?}, ice_servers={}, whitelist_enabled={}, whitelist={}, modpack={}",
            p.max_players, p.host_mc_version, p.host_mc_port,
            p.host_loader, p.host_loader_version,
            p.room_type, p.lobby_id,
            p.ice_servers.len(), p.whitelist_enabled, p.whitelist.len(),
            p.modpack.as_ref().map(|m| format!("{}({}:{})", m.source, m.project_id, m.file_id)).unwrap_or_else(|| "none".to_string())
        );
        let req = CreateRoomRequest {
            sdp_offer: p.sdp_offer,
            ice_candidates: p.ice_candidates,
            max_players: p.max_players,
            password: p.password,
            stun_servers: p.stun_servers,
            ice_servers: p.ice_servers,
            host_mc_version: p.host_mc_version,
            host_mc_port: p.host_mc_port,
            host_loader: p.host_loader,
            host_loader_version: p.host_loader_version,
            room_type: p.room_type,
            lobby_id: p.lobby_id,
            whitelist_enabled: p.whitelist_enabled,
            whitelist: p.whitelist,
            modpack: p.modpack,
        };
        let result = client.signaling_create_room(&creds, &req).await
            .map_err(|e| {
                log_error!("[Online] room_create 失败: {}", e);
                e.to_string()
            })?;
        if let Some(ref data) = result.data {
            log_info!("[Online] 房间创建成功: room_code={}", data.room_code);
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

fn register_join_room(d: &mut Dispatcher) {
    d.register(
        "room_join",
        handler!(state, _app, params, {
            let p: JoinRoomParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!("[Online] room_join: code={}", p.room_code);
            let result = client
                .signaling_join_room(&creds, &p.room_code, &p.password)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_join 失败: {}", e);
                    e.to_string()
                })?;
            if let Some(ref data) = result.data {
                log_info!(
                    "[Online] 加入房间成功: participant_id={}, virtual_ip={}",
                    data.participant_id,
                    data.player_virtual_ip
                );
            }
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_keepalive(d: &mut Dispatcher) {
    d.register(
        "room_keepalive",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] room_keepalive: code={}", p.room_code);
            let result = client
                .signaling_keepalive(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_keepalive 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

/// 房主独占接口：拉取服务端 TURN 服务器列表（阶段三子任务 7）
///
/// 服务端经负载与启用状态过滤后返回 TURN 服务器数组，
/// 房主拉取后通过 P2P DataChannel 广播 `TurnServers` 控制消息给所有参与者。
fn register_get_turn_servers(d: &mut Dispatcher) {
    d.register("room_get_turn", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = super::load_creds(&state).await?;
        let client = super::make_client(&state).await;
        log_debug!("[Online] room_get_turn: code={}", p.room_code);
        let result = client.signaling_get_turn_servers(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_get_turn 失败: {}", e);
                e.to_string()
            })?;
        if let Some(ref data) = result.data {
            log_info!(
                "[Online] 房主拉取 TURN 服务器: enabled={}, servers={}, total_load={}, threshold={}",
                data.enabled, data.servers.len(), data.current_total_load, data.load_threshold
            );
        }
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_leave_room(d: &mut Dispatcher) {
    d.register(
        "room_leave",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!("[Online] room_leave: code={}", p.room_code);
            let result = client
                .signaling_leave_room(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_leave 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}
