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
    /// 厂商远端隧道自增 ID（从厂商 API 导入时记录，用于同步面板判断已导入；本地自建隧道为空）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_tunnel_id: Option<String>,
    /// 厂商远端隧道真实 name（真实隧道标识，非自增 id）。
    /// config 接口查询用该值（如 Lolia `/user/frpc/config?tunnel=<name>`），
    /// 生成 frpc 配置的 `[[proxies]] name` 也用该值。从厂商 API 导入时记录。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_tunnel_name: Option<String>,
    /// 厂商 config 接口返回的完整配置，导入时原样保存，启动时直接复用。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_config: Option<String>,
    /// 带宽限制（如 "4MB"），写入 `[proxies.transport] bandwidthLimit`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bandwidth_limit: Option<String>,
    /// 带宽限制模式（如 "server"），写入 `[proxies.transport] bandwidthLimitMode`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bandwidth_limit_mode: Option<String>,
    /// Proxy 传输加密
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_use_encryption: Option<bool>,
    /// Proxy 传输压缩
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_use_compression: Option<bool>,
    /// Proxy 协议版本
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_protocol_version: Option<String>,
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
