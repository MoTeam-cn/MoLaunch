//! 信令 action 管理器（房间创建/加入/退出/踢人/保活等）
//!
//! 由 `online_manager::DISPATCHER` 调用 `register_signaling_actions` 注册全部信令 action。
//! 拆分到独立模块避免 `online_manager.rs` 超过 500 行。命名遵循项目 `xxx_manager.rs` 惯例。
//!
//! 所有信令 action 统一流程：
//! 1. 从 params 解析参数
//! 2. 加载设备凭证（需已注册）
//! 3. 调用 `OnlineClient` 对应的 `signaling_*` 方法
//! 4. 返回业务数据

use serde::Deserialize;

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::signaling::CreateRoomRequest;
use crate::minecraft::online::signaling::UploadParticipantOfferRequest;
use crate::minecraft::online::storage::OnlineStorage;
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;

// ============================================================
// 参数结构体
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomParams {
    pub sdp_offer: String,
    #[serde(default)]
    pub ice_candidates: Vec<String>,
    #[serde(default = "default_max_players")]
    pub max_players: u32,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub stun_servers: Vec<String>,
    #[serde(default)]
    pub host_mc_version: String,
    #[serde(default)]
    pub host_mc_port: u16,
}

fn default_max_players() -> u32 {
    8
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomCodeParams {
    pub room_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomParams {
    pub room_code: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAnswerParams {
    pub room_code: String,
    pub participant_id: String,
    pub sdp_answer: String,
    #[serde(default)]
    pub ice_candidates: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmParams {
    pub room_code: String,
    pub participant_id: String,
    pub accepted: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KickParams {
    pub room_code: String,
    pub participant_id: String,
    #[serde(default)]
    pub ban_duration_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnbanParams {
    pub room_code: String,
    pub device_pk: String,
}

/// 房主为指定参与者上传 SDP Offer 的参数（mesh 拓扑）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadParticipantOfferParams {
    pub room_code: String,
    pub participant_id: String,
    pub sdp_offer: String,
    #[serde(default)]
    pub ice_candidates: Vec<String>,
}

/// 参与者拉取房主为自己生成的 SDP Offer 的参数（mesh 拓扑）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantOfferParams {
    pub room_code: String,
    pub participant_id: String,
}

// ============================================================
// 辅助函数
// ============================================================

/// 加载设备凭证（需已注册）
async fn load_creds(state: &AppState) -> Result<crate::minecraft::online::storage::DeviceCredentials, String> {
    let storage = OnlineStorage::new(state.sdk.clone());
    let creds = storage
        .load()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "设备未注册，请先注册".to_string())?;
    if !creds.is_registered() {
        return Err("设备未注册，请先注册".to_string());
    }
    if creds.is_token_expired() {
        return Err("JWT 已过期，请重新登录".to_string());
    }
    Ok(creds)
}

/// 创建 OnlineClient
async fn make_client(state: &AppState) -> OnlineClient {
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    OnlineClient::new(&base_url)
}

// ============================================================
// 注册入口
// ============================================================

/// 注册全部信令 action 到 dispatcher
pub fn register_signaling_actions(d: &mut Dispatcher) {
    register_get_stun(d);
    register_create_room(d);
    register_get_room(d);
    register_close_room(d);
    register_join_room(d);
    register_submit_answer(d);
    register_list_answers(d);
    register_confirm(d);
    register_keepalive(d);
    register_leave_room(d);
    register_kick(d);
    register_unban(d);
    register_list_participants(d);
    register_upload_participant_offer(d);
    register_fetch_participant_offer(d);
}

// ============================================================
// 各 action 注册
// ============================================================

fn register_get_stun(d: &mut Dispatcher) {
    d.register("room_get_stun", handler!(state, _app, _params, {
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_get_stun");
        let result = client.signaling_get_stun(&creds).await
            .map_err(|e| {
                log_error!("[Online] room_get_stun 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_create_room(d: &mut Dispatcher) {
    d.register("room_create", handler!(state, _app, params, {
        let p: CreateRoomParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_create: max_players={}, mc_version={}, mc_port={}",
            p.max_players, p.host_mc_version, p.host_mc_port
        );
        let req = CreateRoomRequest {
            sdp_offer: p.sdp_offer,
            ice_candidates: p.ice_candidates,
            max_players: p.max_players,
            password: p.password,
            stun_servers: p.stun_servers,
            host_mc_version: p.host_mc_version,
            host_mc_port: p.host_mc_port,
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
    d.register("room_get", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_get: code={}", p.room_code);
        let result = client.signaling_get_room(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_get 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_close_room(d: &mut Dispatcher) {
    d.register("room_close", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Online] room_close: code={}", p.room_code);
        let result = client.signaling_close_room(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_close 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_join_room(d: &mut Dispatcher) {
    d.register("room_join", handler!(state, _app, params, {
        let p: JoinRoomParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Online] room_join: code={}", p.room_code);
        let result = client.signaling_join_room(&creds, &p.room_code, &p.password).await
            .map_err(|e| {
                log_error!("[Online] room_join 失败: {}", e);
                e.to_string()
            })?;
        if let Some(ref data) = result.data {
            log_info!(
                "[Online] 加入房间成功: participant_id={}, virtual_ip={}",
                data.participant_id, data.player_virtual_ip
            );
        }
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_submit_answer(d: &mut Dispatcher) {
    d.register("room_submit_answer", handler!(state, _app, params, {
        let p: SubmitAnswerParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!(
            "[Online] room_submit_answer: code={}, participant={}",
            p.room_code, p.participant_id
        );
        let result = client
            .signaling_submit_answer(&creds, &p.room_code, &p.participant_id, &p.sdp_answer, &p.ice_candidates)
            .await
            .map_err(|e| {
                log_error!("[Online] room_submit_answer 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_list_answers(d: &mut Dispatcher) {
    d.register("room_list_answers", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        let result = client.signaling_list_answers(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_list_answers 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_confirm(d: &mut Dispatcher) {
    d.register("room_confirm", handler!(state, _app, params, {
        let p: ConfirmParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_confirm: code={}, participant={}, accepted={}",
            p.room_code, p.participant_id, p.accepted
        );
        let result = client.signaling_confirm(&creds, &p.room_code, &p.participant_id, p.accepted).await
            .map_err(|e| {
                log_error!("[Online] room_confirm 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_keepalive(d: &mut Dispatcher) {
    d.register("room_keepalive", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_keepalive: code={}", p.room_code);
        let result = client.signaling_keepalive(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_keepalive 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_leave_room(d: &mut Dispatcher) {
    d.register("room_leave", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Online] room_leave: code={}", p.room_code);
        let result = client.signaling_leave_room(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_leave 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_kick(d: &mut Dispatcher) {
    d.register("room_kick", handler!(state, _app, params, {
        let p: KickParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_kick: code={}, participant={}, ban={:?}",
            p.room_code, p.participant_id, p.ban_duration_seconds
        );
        let result = client
            .signaling_kick(&creds, &p.room_code, &p.participant_id, p.ban_duration_seconds)
            .await
            .map_err(|e| {
                log_error!("[Online] room_kick 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_unban(d: &mut Dispatcher) {
    d.register("room_unban", handler!(state, _app, params, {
        let p: UnbanParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Online] room_unban: code={}, device_pk={}", p.room_code, p.device_pk);
        let result = client.signaling_unban(&creds, &p.room_code, &p.device_pk).await
            .map_err(|e| {
                log_error!("[Online] room_unban 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_list_participants(d: &mut Dispatcher) {
    d.register("room_list_participants", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_list_participants: code={}", p.room_code);
        let result = client.signaling_list_participants(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_list_participants 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_upload_participant_offer(d: &mut Dispatcher) {
    d.register("room_upload_participant_offer", handler!(state, _app, params, {
        let p: UploadParticipantOfferParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!(
            "[Online] room_upload_participant_offer: code={}, participant={}",
            p.room_code, p.participant_id
        );
        let req = UploadParticipantOfferRequest {
            sdp_offer: p.sdp_offer,
            ice_candidates: p.ice_candidates,
        };
        let result = client
            .signaling_upload_participant_offer(&creds, &p.room_code, &p.participant_id, &req)
            .await
            .map_err(|e| {
                log_error!("[Online] room_upload_participant_offer 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_fetch_participant_offer(d: &mut Dispatcher) {
    d.register("room_fetch_participant_offer", handler!(state, _app, params, {
        let p: ParticipantOfferParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!(
            "[Online] room_fetch_participant_offer: code={}, participant={}",
            p.room_code, p.participant_id
        );
        let result = client
            .signaling_fetch_participant_offer(&creds, &p.room_code, &p.participant_id)
            .await
            .map_err(|e| {
                log_error!("[Online] room_fetch_participant_offer 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}
