//! 隧道与日志类型

use serde::{Deserialize, Serialize};

// 隧道相关类型
/// 隧道类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    Tcp,
    Udp,
}

/// 隧道运行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Running,
    Stopped,
}

/// 隧道配置（持久化到 tunnels.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tunnel {
    /// 隧道唯一 ID
    pub id: String,
    /// 隧道名称（用户自定义）
    pub name: String,
    /// 所属厂商 ID
    pub provider_id: String,
    /// 隧道类型
    pub tunnel_type: TunnelType,
    /// 本地 IP（默认 127.0.0.1）
    pub local_ip: String,
    /// 本地端口（如 25565）
    pub local_port: u16,
    /// Frp 服务器地址
    pub server_addr: String,
    /// Frp 服务器端口
    pub server_port: u16,
    /// 远程端口（tcp/udp 类型必填）
    pub remote_port: u16,
    /// Frp 服务器鉴权 token（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// 是否启用 TLS
    #[serde(default)]
    pub use_tls: bool,
    /// 创建时间（Unix 毫秒）
    pub created_at: u64,
}

/// 隧道 + 运行状态（返回给前端）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelWithStatus {
    #[serde(flatten)]
    pub tunnel: Tunnel,
    /// 当前运行状态
    pub status: TunnelStatus,
    /// 运行中的进程 PID（status=running 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

// 日志文件信息
/// 日志文件信息（list_log_files 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFileInfo {
    pub tunnel_id: String,
    pub size_bytes: u64,
    pub modified_at: u64,
}

/// 日志文件内容（read_log_file 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFileContent {
    pub lines: Vec<String>,
    pub has_more: bool,
}
