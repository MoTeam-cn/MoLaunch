use serde::{Deserialize, Serialize};

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
