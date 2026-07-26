//! P2P 联机信令接口客户端
//!
//! 对接 api-server `/v1/signaling/*` 接口，实现房间创建/加入/退出/踢人/保活等。
//!
//! 接口参考：`api-server/docs/signaling.md`
//!
//! 阶段一仅声明类型与接口签名，阶段二填充实现。

use serde::{Deserialize, Serialize};

use super::client::{BusinessResult, ClientError, OnlineClient};
use super::storage::DeviceCredentials;

// ============================== 请求/响应类型 ==============================

/// STUN 服务器列表响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StunServersResponse {
    pub servers: Vec<String>,
}

/// 创建房间请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateRoomRequest {
    pub sdp_offer: String,
    pub ice_candidates: Vec<String>,
    pub max_players: u32,
    pub password: String,
    pub stun_servers: Vec<String>,
    /// 房主 MC 版本（客户端扩展字段，由启动器探测本地 MC 端口后填入）
    pub host_mc_version: String,
    /// 房主 MC 端口（客户端扩展字段，启动器探测本地 Java 进程端口后填入）
    pub host_mc_port: u16,
}

/// 创建房间响应
///
/// `rename_all = "camelCase"`：序列化输出 camelCase 给前端
/// `alias`：反序列化时接受 api-server 返回的 snake_case
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomResponse {
    #[serde(alias = "room_code")]
    pub room_code: String,
    #[serde(alias = "host_virtual_ip")]
    pub host_virtual_ip: String,
    pub subnet: String,
    #[serde(alias = "created_at")]
    pub created_at: u64,
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
}

/// 房间公开信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfoResponse {
    #[serde(alias = "room_code")]
    pub room_code: String,
    #[serde(alias = "host_device_pk")]
    pub host_device_pk: String,
    #[serde(alias = "max_players")]
    pub max_players: u32,
    #[serde(alias = "player_count")]
    pub player_count: u32,
    #[serde(alias = "has_password")]
    pub has_password: bool,
    #[serde(alias = "stun_servers")]
    pub stun_servers: Vec<String>,
    pub status: String,
    #[serde(alias = "created_at")]
    pub created_at: u64,
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
    /// 房主 MC 版本（客户端扩展字段，由创建房间时上报）
    #[serde(default, alias = "host_mc_version")]
    pub host_mc_version: String,
    /// 房主 MC 端口（客户端扩展字段）
    #[serde(default, alias = "host_mc_port")]
    pub host_mc_port: u16,
}

/// 加入房间响应
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomResponse {
    #[serde(alias = "participant_id")]
    pub participant_id: String,
    #[serde(alias = "host_sdp_offer")]
    pub host_sdp_offer: String,
    #[serde(alias = "host_ice_candidates")]
    pub host_ice_candidates: Vec<String>,
    #[serde(alias = "stun_servers")]
    pub stun_servers: Vec<String>,
    #[serde(alias = "player_virtual_ip")]
    pub player_virtual_ip: String,
    pub subnet: String,
}

/// 待确认 Answer
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAnswer {
    #[serde(alias = "participant_id")]
    pub participant_id: String,
    #[serde(alias = "device_pk")]
    pub device_pk: String,
    #[serde(alias = "sdp_answer")]
    pub sdp_answer: String,
    #[serde(alias = "ice_candidates")]
    pub ice_candidates: Vec<String>,
    #[serde(alias = "player_virtual_ip")]
    pub player_virtual_ip: String,
    #[serde(alias = "joined_at")]
    pub joined_at: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListAnswersResponse {
    pub answers: Vec<PendingAnswer>,
}

/// 参与者信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantInfo {
    #[serde(alias = "participant_id")]
    pub participant_id: String,
    #[serde(alias = "device_pk")]
    pub device_pk: String,
    #[serde(alias = "virtual_ip")]
    pub virtual_ip: String,
    pub status: String,
    #[serde(alias = "joined_at")]
    pub joined_at: u64,
    #[serde(alias = "confirmed_at")]
    pub confirmed_at: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListParticipantsResponse {
    pub participants: Vec<ParticipantInfo>,
}

/// keepalive 响应
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepaliveResponse {
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
    #[serde(alias = "server_time")]
    pub server_time: u64,
}

// ============================== 客户端扩展方法 ==============================

impl OnlineClient {
    /// 获取 STUN 服务器列表（GET /v1/signaling/stun）
    pub async fn signaling_get_stun(
        &self,
        creds: &DeviceCredentials,
    ) -> Result<BusinessResult<StunServersResponse>, ClientError> {
        self.call_v1::<StunServersResponse>(creds, "GET", "/v1/signaling/stun", None, false)
            .await
    }

    /// 创建房间（POST /v1/signaling/rooms）
    pub async fn signaling_create_room(
        &self,
        creds: &DeviceCredentials,
        req: &CreateRoomRequest,
    ) -> Result<BusinessResult<CreateRoomResponse>, ClientError> {
        let body = serde_json::to_value(req)?;
        self.call_v1::<CreateRoomResponse>(creds, "POST", "/v1/signaling/rooms", Some(&body), true)
            .await
    }

    /// 查询房间公开信息（GET /v1/signaling/rooms/{code}）
    pub async fn signaling_get_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<RoomInfoResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}", room_code);
        self.call_v1::<RoomInfoResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 关闭房间（DELETE /v1/signaling/rooms/{code}）
    pub async fn signaling_close_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}", room_code);
        self.call_v1::<serde_json::Value>(creds, "DELETE", &path, None, true)
            .await
    }

    /// 加入房间（POST /v1/signaling/rooms/{code}/join）
    pub async fn signaling_join_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        password: &str,
    ) -> Result<BusinessResult<JoinRoomResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/join", room_code);
        let body = serde_json::json!({ "password": password });
        self.call_v1::<JoinRoomResponse>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 提交 SDP Answer（POST /v1/signaling/rooms/{code}/answer）
    pub async fn signaling_submit_answer(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        sdp_answer: &str,
        ice_candidates: &[String],
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/answer", room_code);
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
        let path = format!("/v1/signaling/rooms/{}/answers", room_code);
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
        let path = format!("/v1/signaling/rooms/{}/confirm", room_code);
        let body = serde_json::json!({
            "participant_id": participant_id,
            "accepted": accepted,
        });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 房主保活（POST /v1/signaling/rooms/{code}/keepalive）
    pub async fn signaling_keepalive(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<KeepaliveResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/keepalive", room_code);
        self.call_v1::<KeepaliveResponse>(creds, "POST", &path, None, true)
            .await
    }

    /// 退出房间（DELETE /v1/signaling/rooms/{code}/participants/me）
    pub async fn signaling_leave_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/participants/me", room_code);
        self.call_v1::<serde_json::Value>(creds, "DELETE", &path, None, true)
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
        let path = format!("/v1/signaling/rooms/{}/kick", room_code);
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
        let path = format!("/v1/signaling/rooms/{}/unban", room_code);
        let body = serde_json::json!({ "device_pk": device_pk });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 查询参与者列表（GET /v1/signaling/rooms/{code}/participants）
    pub async fn signaling_list_participants(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<ListParticipantsResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/participants", room_code);
        self.call_v1::<ListParticipantsResponse>(creds, "GET", &path, None, false)
            .await
    }
}
