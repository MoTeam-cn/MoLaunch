//! 账号信息 DTO（微软 / 离线）

use serde::Serialize;

/// 已存储的微软账号信息
#[derive(Debug, Clone, Serialize)]
pub struct MsAccountInfo {
    pub username: String,
    pub uuid: String,
    pub expires_at: u64,
    pub is_expired: bool,
    /// 续期中：token 已过期且静默续期失败（refresh_token 失效等），前端据此显示「续期中」而非「已过期」
    #[serde(default, skip_serializing_if = "is_false")]
    pub refreshing: bool,
}

/// serde 辅助：bool 默认值（false 时不序列化）
fn is_false(b: &bool) -> bool {
    !*b
}

/// 已存储的离线账号信息
#[derive(Debug, Clone, Serialize)]
pub struct OfflineAccountInfo {
    pub username: String,
    pub uuid: String,
    pub skin: Option<String>,
}