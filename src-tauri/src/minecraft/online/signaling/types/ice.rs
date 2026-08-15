use serde::{Deserialize, Serialize};

/// ICE 服务器条目（兼容旧配置项 `custom_turn_servers` 反序列化）
///
/// TURN 链路随 WebRTC 一并移除后不再产生新业务，仅保留类型供配置读写。
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
