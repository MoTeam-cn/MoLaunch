use serde::{Deserialize, Serialize};

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
    #[serde(default, alias = "host_offer_ready")]
    pub host_offer_ready: bool,
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
