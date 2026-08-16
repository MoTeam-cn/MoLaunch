use serde::{Deserialize, Serialize};

/// 网络延迟测试请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkLatencyTestParams {
    /// 待测 URL 列表
    pub urls: Vec<String>,
}

/// 网络延迟测试结果
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkLatencyResult {
    pub results: Vec<LatencyItem>,
}

/// 单个 URL 的延迟测试条目
#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyItem {
    pub url: String,
    /// 延迟（毫秒），失败时为 None
    pub latency_ms: Option<u64>,
    /// HTTP 状态码（如 200），失败时为 0
    pub status_code: u16,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 服务器状态检测请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPingParams {
    pub host: String,
    pub port: u16,
}

/// 服务器状态检测结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPingResult {
    /// 服务器 MOTD（纯文本，已从 JSON/section 符号中提取）
    pub motd: String,
    /// 服务器 MOTD 原始文本（保留 § 格式化代码，供前端解析为彩色显示）
    pub motd_raw: String,
    /// 当前在线人数
    pub online: i32,
    /// 最大人数
    pub max: i32,
    /// 服务器版本（如 "1.20.4"）
    pub version: String,
    /// 延迟（毫秒）
    pub latency_ms: u64,
    /// Favicon（base64 data URI），无则为 None
    pub favicon: Option<String>,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// TCP 端口连通性检测请求参数
///
/// 用于 Frp 等非 Minecraft 协议服务的端口可达性检查：
/// 仅做 TCP 三次握手，不发送任何应用层数据，3 秒超时。
/// 与 `ServerPingParams`（SLP 协议）的区别：本检测不依赖应用层协议，
/// 适用于 frps / 数据库 / 任意 TCP 服务。
#[derive(Debug, Serialize, Deserialize)]
pub struct TcpCheckParams {
    pub host: String,
    pub port: u16,
}

/// TCP 端口连通性检测结果
#[derive(Debug, Serialize, Deserialize)]
pub struct TcpCheckResult {
    /// 是否可连接
    pub reachable: bool,
    /// TCP 握手耗时（毫秒），失败时为 0
    pub latency_ms: u64,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 地址延迟测试目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressTarget {
    /// 显示名（如「南京」），缺省用 host
    #[serde(default)]
    pub name: Option<String>,
    /// 目标主机（域名或 IP）
    pub host: String,
    /// 目标端口（ping 协议忽略）
    pub port: u16,
    /// 测延迟协议：tcp（默认，TCP 握手）/ udp（UDP 探针）/ ping（ICMP，系统 ping）
    #[serde(default = "default_latency_protocol")]
    pub protocol: String,
}

fn default_latency_protocol() -> String {
    "tcp".to_string()
}

/// 地址延迟测试请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressLatencyTestParams {
    /// 待测目标列表
    pub targets: Vec<AddressTarget>,
}

/// 地址延迟测试单条结果
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressLatencyItem {
    pub name: Option<String>,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    /// 是否可达
    pub reachable: bool,
    /// 延迟（毫秒），失败时为 0
    pub latency_ms: u64,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 地址延迟测试结果
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressLatencyResult {
    pub results: Vec<AddressLatencyItem>,
}

/// 本机监听端口条目
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenPortInfo {
    /// 本地绑定的完整地址（如 "0.0.0.0:7000" / "127.0.0.1:25565"）
    pub local_addr: String,
    /// 端口号
    pub port: u16,
    /// 协议：tcp / udp
    pub protocol: String,
    /// 占用该端口的进程名（拿不到时为 None）
    pub process_name: Option<String>,
    /// 占用该端口的进程 PID（拿不到时为 None）
    pub pid: Option<u32>,
}

/// 列出本机监听端口结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ListOpenPortsResult {
    pub ports: Vec<OpenPortInfo>,
}

/// 正版玩家皮肤获取请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct SkinFetchParams {
    /// 玩家名（不区分大小写，正版 API 会规范化为实际注册名）
    pub name: String,
}

/// 正版玩家皮肤获取结果
#[derive(Debug, Serialize, Deserialize)]
pub struct SkinFetchResult {
    /// 玩家名（正版 API 返回的规范化名称）
    pub name: String,
    /// 玩家 UUID（32 位十六进制，无连字符）
    pub uuid: String,
    /// 皮肤模型："slim"（Alex 细手臂）| "classic"（Steve 粗手臂）
    pub skin_model: String,
    /// 皮肤图片地址
    pub skin_url: String,
    /// 皮肤 PNG（base64 data URI，供前端直接预览）
    pub skin_image: String,
    /// 披风地址（无披风时为 None）
    pub cape_url: Option<String>,
    /// 披风 PNG（base64 data URI，无披风时为 None）
    pub cape_image: Option<String>,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 保存皮肤图片请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct SkinSaveImageParams {
    /// 保存路径（含文件名，如 D:/skin/Steve.png）
    pub save_path: String,
    /// 图片 base64（不含 data URI 前缀）
    pub image_base64: String,
}
