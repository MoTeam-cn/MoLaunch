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

/// ICE 服务器条目（对应浏览器 `RTCIceServer` 接口）
///
/// 阶段三子任务 7：STUN 服务器仅填 `urls`，TURN 服务器三个字段都填。
/// 序列化为 JSON 后可直接作为 `RTCPeerConnection` 构造参数的 `iceServers` 数组元素。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IceServerEntry {
    pub urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

impl IceServerEntry {
    /// 从 STUN URL 字符串构造（username/credential 为 None）
    pub fn from_stun_url(url: String) -> Self {
        Self {
            urls: vec![url],
            username: None,
            credential: None,
        }
    }

    /// 从 STUN URL 数组构造（username/credential 为 None）
    pub fn from_stun_urls(urls: Vec<String>) -> Self {
        Self {
            urls,
            username: None,
            credential: None,
        }
    }
}

/// STUN 服务器列表响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StunServersResponse {
    pub servers: Vec<String>,
}

/// TURN 服务器列表响应（房主独占接口，阶段三子任务 7）
///
/// 服务端经负载与启用状态过滤后返回 TURN 服务器数组，
/// 房主拉取后通过 P2P DataChannel 广播 `TurnServers` 控制消息给所有参与者。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnServersResponse {
    /// TURN 服务器条目数组（已过滤禁用/超载的服务器）
    pub servers: Vec<IceServerEntry>,
    /// 服务端是否启用 TURN 中转（false 时 servers 必为空，客户端可降级为纯 STUN）
    pub enabled: bool,
    /// 当前所有启用 TURN 的负载之和（供客户端观测服务端压力）
    #[serde(default, alias = "current_total_load")]
    pub current_total_load: u32,
    /// 服务端配置的负载阈值（0 表示不限制）
    #[serde(default, alias = "load_threshold")]
    pub load_threshold: u32,
}

/// 创建房间请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateRoomRequest {
    pub sdp_offer: String,
    pub ice_candidates: Vec<String>,
    pub max_players: u32,
    pub password: String,
    pub stun_servers: Vec<String>,
    /// ICE 服务器列表（新客户端优先，可包含 STUN + TURN 凭据）
    ///
    /// 阶段三子任务 7 新增。为空时服务端使用 `stun_servers` 转换为
    /// `IceServerEntry { urls, username: None, credential: None }` 后落库。
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ice_servers: Vec<IceServerEntry>,
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
    /// ICE 服务器列表（统一承载 STUN + TURN，新客户端优先使用）
    ///
    /// 阶段三子任务 7 新增。客户端应优先使用此字段构造 `RTCPeerConnection`，
    /// 为空时回退使用 `stun_servers`。
    #[serde(default, alias = "ice_servers")]
    pub ice_servers: Vec<IceServerEntry>,
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
    /// ICE 服务器列表（与房主一致，统一承载 STUN + TURN）
    ///
    /// 阶段三子任务 7 新增。客户端应优先使用此字段构造 `RTCPeerConnection`，
    /// 为空时回退使用 `stun_servers`。
    #[serde(default, alias = "ice_servers")]
    pub ice_servers: Vec<IceServerEntry>,
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
    /// 房主是否已为该参与者生成 SDP Offer（mesh 拓扑，true 表示 offer 已就绪）
    #[serde(default)]
    pub host_offer_ready: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListParticipantsResponse {
    pub participants: Vec<ParticipantInfo>,
}

/// 房主为指定参与者上传 SDP Offer 的请求体（mesh 拓扑）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default)]
    pub sdp_offer: String,
    /// ICE Candidates 数组（未就绪时为空数组）
    #[serde(default)]
    pub ice_candidates: Vec<String>,
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

    /// 房主独占接口：拉取服务端 TURN 服务器列表（GET /v1/signaling/rooms/{code}/turn）
    ///
    /// 阶段三子任务 7：服务端经负载与启用状态过滤后返回 TURN 服务器数组，
    /// 房主拉取后通过 P2P DataChannel 广播 `TurnServers` 控制消息给所有参与者。
    pub async fn signaling_get_turn_servers(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<TurnServersResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/turn", room_code);
        self.call_v1::<TurnServersResponse>(creds, "GET", &path, None, false)
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

    /// 房主为指定参与者上传 SDP Offer（PUT /v1/signaling/rooms/{code}/participants/{participant_id}/offer）
    ///
    /// mesh 拓扑：房主轮询 participants 列表发现 `host_offer_ready=false` 的 `joined` 参与者时，
    /// 为其创建独立 PeerConnection + DataChannel + Offer，然后调用本接口上传。
    pub async fn signaling_upload_participant_offer(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        req: &UploadParticipantOfferRequest,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!(
            "/v1/signaling/rooms/{}/participants/{}/offer",
            room_code, participant_id
        );
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
        let path = format!(
            "/v1/signaling/rooms/{}/participants/{}/offer",
            room_code, participant_id
        );
        self.call_v1::<ParticipantOfferResponse>(creds, "GET", &path, None, false)
            .await
    }
}
