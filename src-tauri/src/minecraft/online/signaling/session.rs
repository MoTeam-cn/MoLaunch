//! 会话管理接口：Answer 提交/确认、踢人/解封/封禁列表、参与者列表、mesh Offer 上传/拉取。
//!
//! 同时定义封禁、Offer 上传/拉取相关的请求/响应类型。

use serde::{Deserialize, Serialize};

use super::types::{ListAnswersResponse, ListParticipantsResponse};
use crate::api_paths;
use crate::minecraft::online::client::{BusinessResult, ClientError, OnlineClient};
use crate::minecraft::online::storage::DeviceCredentials;

// 封禁 / Offer 类型

/// 房间封禁记录（对应 api-server `RoomBan`）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomBan {
    pub id: String,
    #[serde(alias = "room_code")]
    pub room_code: String,
    #[serde(alias = "device_pk")]
    pub device_pk: String,
    /// 0=永久封禁；>0=解封 Unix 秒时间戳
    #[serde(alias = "banned_until")]
    pub banned_until: i64,
    #[serde(alias = "created_at")]
    pub created_at: i64,
}

/// 封禁列表响应（对应 api-server `ListBansResponse`）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBansResponse {
    /// 当前有效封禁记录（永久 + 未过期临时）
    pub bans: Vec<RoomBan>,
    /// 服务端当前 Unix 秒，便于客户端计算剩余封禁时长
    #[serde(alias = "server_time")]
    pub server_time: i64,
}

/// 房主为指定参与者上传 SDP Offer 的请求体（mesh 拓扑）
///
/// 无 `rename_all`：此结构体仅 Serialize（客户端→服务端），服务端期望 snake_case。
/// 若加 `rename_all = "camelCase"` 会序列化为 `sdpOffer`/`iceCandidates`，服务端反序列化失败。
#[derive(Debug, Clone, Serialize)]
pub struct UploadParticipantOfferRequest {
    pub sdp_offer: String,
    pub ice_candidates: Vec<String>,
}

/// 参与者拉取房主为自己生成的 SDP Offer 的响应（mesh 拓扑）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantOfferResponse {
    /// Offer 是否已就绪（等价于 sdp_offer 非空）
    pub ready: bool,
    /// SDP Offer（未就绪时为空字符串）
    #[serde(default, alias = "sdp_offer")]
    pub sdp_offer: String,
    /// ICE Candidates 数组（未就绪时为空数组）
    #[serde(default, alias = "ice_candidates")]
    pub ice_candidates: Vec<String>,
}

// OnlineClient 扩展方法

impl OnlineClient {
    /// 提交 SDP Answer（POST /v1/signaling/rooms/{code}/answer）
    pub async fn signaling_submit_answer(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        sdp_answer: &str,
        ice_candidates: &[String],
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_ANSWER.replace("{room_code}", room_code);
        let body = serde_json::json!({
            "participant_id": participant_id,
            "sdp_answer": sdp_answer,
            "ice_candidates": ice_candidates,
        });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 拉取待确认 Answer 列表（GET /v1/signaling/rooms/{code}/answers）
    pub async fn signaling_list_answers(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<ListAnswersResponse>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_ANSWERS.replace("{room_code}", room_code);
        self.call_v1::<ListAnswersResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 确认/拒绝连接（POST /v1/signaling/rooms/{code}/confirm）
    pub async fn signaling_confirm(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        accepted: bool,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_CONFIRM.replace("{room_code}", room_code);
        let body = serde_json::json!({
            "participant_id": participant_id,
            "accepted": accepted,
        });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 踢出参与者（POST /v1/signaling/rooms/{code}/kick）
    pub async fn signaling_kick(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        ban_duration_seconds: Option<u64>,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_KICK.replace("{room_code}", room_code);
        let body = serde_json::json!({
            "participant_id": participant_id,
            "ban_duration_seconds": ban_duration_seconds,
        });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 解封参与者（POST /v1/signaling/rooms/{code}/unban）
    pub async fn signaling_unban(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        device_pk: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_UNBAN.replace("{room_code}", room_code);
        let body = serde_json::json!({ "device_pk": device_pk });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 查询房间封禁列表（GET /v1/signaling/rooms/{code}/bans，仅房主）
    ///
    /// 返回当前有效的封禁记录（永久 + 未过期临时），已过期的临时封禁不返回。
    /// 同时返回服务端当前时间 `server_time`，便于客户端计算剩余封禁时长。
    pub async fn signaling_list_bans(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<ListBansResponse>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_BANS.replace("{room_code}", room_code);
        self.call_v1::<ListBansResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 查询参与者列表（GET /v1/signaling/rooms/{code}/participants）
    pub async fn signaling_list_participants(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<ListParticipantsResponse>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_PARTICIPANTS.replace("{room_code}", room_code);
        self.call_v1::<ListParticipantsResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 房主为指定参与者上传 SDP Offer（PUT /v1/signaling/rooms/{code}/participants/{participant_id}/offer）
    ///
    /// mesh 拓扑：授权前置——房主在「加入申请」中确认接受（status=confirmed）后，
    /// 轮询 participants 列表发现 `host_offer_ready=false` 的 confirmed 参与者时，
    /// 为其创建独立 PeerConnection + DataChannel + Offer，然后调用本接口上传。
    pub async fn signaling_upload_participant_offer(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        req: &UploadParticipantOfferRequest,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_PARTICIPANT_OFFER
            .replace("{room_code}", room_code)
            .replace("{participant_id}", participant_id);
        let body = serde_json::to_value(req)?;
        self.call_v1::<serde_json::Value>(creds, "PUT", &path, Some(&body), true)
            .await
    }

    /// 参与者拉取房主为自己生成的 SDP Offer（GET /v1/signaling/rooms/{code}/participants/{participant_id}/offer）
    ///
    /// mesh 拓扑：参与者 join 后轮询本接口，`ready=false` 表示房主尚未生成 Offer。
    pub async fn signaling_fetch_participant_offer(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
    ) -> Result<BusinessResult<ParticipantOfferResponse>, ClientError> {
        let path = api_paths::SIGNALING_ROOM_PARTICIPANT_OFFER
            .replace("{room_code}", room_code)
            .replace("{participant_id}", participant_id);
        self.call_v1::<ParticipantOfferResponse>(creds, "GET", &path, None, false)
            .await
    }
}
