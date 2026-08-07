//! 会话管理 action 注册：Answer 提交/确认、踢人/解封/封禁列表、参与者列表、mesh Offer 上传/拉取。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::signaling::UploadParticipantOfferRequest;
use crate::utils::dispatcher::Dispatcher;

use super::{
    ConfirmParams, KickParams, ParticipantOfferParams, RoomCodeParams, SubmitAnswerParams,
    UnbanParams, UploadParticipantOfferParams,
};

/// 注册会话/封禁/Offer 相关 action
pub fn register(d: &mut Dispatcher) {
    register_submit_answer(d);
    register_list_answers(d);
    register_confirm(d);
    register_kick(d);
    register_unban(d);
    register_list_bans(d);
    register_list_participants(d);
    register_upload_participant_offer(d);
    register_fetch_participant_offer(d);
}

fn register_submit_answer(d: &mut Dispatcher) {
    d.register(
        "room_submit_answer",
        handler!(state, _app, params, {
            let p: SubmitAnswerParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!(
                "[Online] room_submit_answer: code={}, participant={}",
                p.room_code,
                p.participant_id
            );
            let result = client
                .signaling_submit_answer(
                    &creds,
                    &p.room_code,
                    &p.participant_id,
                    &p.sdp_answer,
                    &p.ice_candidates,
                )
                .await
                .map_err(|e| {
                    log_error!("[Online] room_submit_answer 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_list_answers(d: &mut Dispatcher) {
    d.register(
        "room_list_answers",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            let result = client
                .signaling_list_answers(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_list_answers 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_confirm(d: &mut Dispatcher) {
    d.register(
        "room_confirm",
        handler!(state, _app, params, {
            let p: ConfirmParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!(
                "[Online] room_confirm: code={}, participant={}, accepted={}",
                p.room_code,
                p.participant_id,
                p.accepted
            );
            let result = client
                .signaling_confirm(&creds, &p.room_code, &p.participant_id, p.accepted)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_confirm 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_kick(d: &mut Dispatcher) {
    d.register(
        "room_kick",
        handler!(state, _app, params, {
            let p: KickParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!(
                "[Online] room_kick: code={}, participant={}, ban={:?}",
                p.room_code,
                p.participant_id,
                p.ban_duration_seconds
            );
            let result = client
                .signaling_kick(
                    &creds,
                    &p.room_code,
                    &p.participant_id,
                    p.ban_duration_seconds,
                )
                .await
                .map_err(|e| {
                    log_error!("[Online] room_kick 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_unban(d: &mut Dispatcher) {
    d.register(
        "room_unban",
        handler!(state, _app, params, {
            let p: UnbanParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!(
                "[Online] room_unban: code={}, device_pk={}",
                p.room_code,
                p.device_pk
            );
            let result = client
                .signaling_unban(&creds, &p.room_code, &p.device_pk)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_unban 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_list_bans(d: &mut Dispatcher) {
    d.register(
        "room_list_bans",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] room_list_bans: code={}", p.room_code);
            let result = client
                .signaling_list_bans(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_list_bans 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_list_participants(d: &mut Dispatcher) {
    d.register(
        "room_list_participants",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] room_list_participants: code={}", p.room_code);
            let result = client
                .signaling_list_participants(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_list_participants 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_upload_participant_offer(d: &mut Dispatcher) {
    d.register(
        "room_upload_participant_offer",
        handler!(state, _app, params, {
            let p: UploadParticipantOfferParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!(
                "[Online] room_upload_participant_offer: code={}, participant={}",
                p.room_code,
                p.participant_id
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
        }),
    );
}

fn register_fetch_participant_offer(d: &mut Dispatcher) {
    d.register(
        "room_fetch_participant_offer",
        handler!(state, _app, params, {
            let p: ParticipantOfferParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!(
                "[Online] room_fetch_participant_offer: code={}, participant={}",
                p.room_code,
                p.participant_id
            );
            let result = client
                .signaling_fetch_participant_offer(&creds, &p.room_code, &p.participant_id)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_fetch_participant_offer 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}
