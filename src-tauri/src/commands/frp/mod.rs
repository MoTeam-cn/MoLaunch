//! Frp 内网穿透命令模块（编排层）
//!
//! 厂商存放于 `<base_dir>/providers/<provider_id>/`，隧道配置持久化于
//! `<base_dir>/frp/tunnels.json`，frpc 日志写入 `<base_dir>/frp/logs/`。
//! 子模块按职责拆分：provider（列表/状态/启禁）/ install（安装/卸载）/
//! binary（frpc 二进制下载）/ tunnel（CRUD/配置生成）/ process（进程管理/日志）/ sandbox（校验）。
//! 所有子模块函数由 `utils::frp_manager::dispatch` 统一反序列化参数后调用。

pub mod api_schema;
pub mod auth;
pub mod binary;
pub mod install;
pub mod log_redact;
pub mod process;
pub mod provider;
pub mod sandbox;
pub mod tunnel;

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, State};

/// 统一 Frp 管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::frp_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn frp_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::frp_manager::dispatch(state, app, req).await
}

// ============================================================
// 共享类型
// ============================================================

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

/// 厂商信息（返回给前端）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    /// 是否为内置厂商
    pub builtin: bool,
    /// 认证类型：none / oauth2 / device_code / api_key
    pub auth_type: String,
    /// frpc 二进制是否就绪
    pub frpc_ready: bool,
    /// 是否启用（内置厂商始终 true）
    pub enabled: bool,
    /// frpc 分发方式：bundled / url / system（系统默认厂商专属）
    pub distribution: String,
    /// 厂商主页（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

/// 厂商清单（外部厂商的 manifest.json 反序列化结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// frpc 二进制配置
    pub binary: BinaryConfig,
    /// 认证方式（默认 none）
    #[serde(default)]
    pub auth: AuthConfig,
    /// 网络权限（限制 frpc 可连接的服务器，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_permissions: Option<NetworkPermissions>,
    /// 进程权限（限制厂商认证适配器脚本的执行，可选）
    ///
    /// 对应设计文档 §7.5 认证适配器沙箱。仅当厂商提供自定义认证脚本
    /// （如 Node.js / Python）时启用，命令必须通过 `which_canonical` 解析后
    /// 与 `allowed_commands` 白名单匹配，非 shell 执行，超时默认 30 秒、
    /// 最大 5 分钟，stdout/stderr 各截断到 1MB，工作目录限制在厂商目录内。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_permissions: Option<ProcessPermissions>,
}

/// 进程权限配置（限制厂商认证适配器脚本执行）
///
/// 对应设计文档 §7.5。命令必须通过 `which_canonical` 解析后与白名单匹配，
/// 非 shell 执行防注入，超时默认 30 秒、最大 5 分钟，stdout/stderr 各截断到 1MB。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessPermissions {
    /// 允许执行的命令白名单（如 ["node", "python"]）
    #[serde(default)]
    pub allowed_commands: Vec<String>,
    /// 超时毫秒，默认 30000，最大 300000
    #[serde(default = "default_process_timeout_ms")]
    pub timeout_ms: u64,
}

/// serde 默认值：进程超时 30 秒
fn default_process_timeout_ms() -> u64 {
    30_000
}

/// frpc 二进制分发配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryConfig {
    /// 分发方式：bundled=随厂商包打包 / url=按需下载
    #[serde(default = "default_distribution")]
    pub distribution: String,
    /// distribution=bundled 时：厂商自带 frpc 相对路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// distribution=url 时：下载配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<DownloadConfig>,
}

/// URL 下载配置（distribution=url 时使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadConfig {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    pub allowed_domains: Vec<String>,
    pub target_path: String,
    #[serde(default)]
    pub archive: bool,
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    /// 认证类型：none / oauth2 / device_code / api_key
    #[serde(default = "default_auth_type")]
    #[serde(rename = "type")]
    pub auth_type: String,
    /// OAuth2 配置（type=oauth2 时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<OAuth2Config>,
    /// Device Code 配置（type=device_code 时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<DeviceCodeConfig>,
    /// API Key 配置（type=api_key 时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyConfig>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            auth_type: default_auth_type(),
            oauth2: None,
            device_code: None,
            api_key: None,
        }
    }
}

/// 网络权限配置（限制 frpc 可连接的服务器）
///
/// 对应设计文档 §7.2 配置校验中的网络白名单。当 `allow_custom_server=false` 时，
/// `server_addr` 必须在 `allowed_servers` 白名单内；系统默认厂商始终允许自定义服务器。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPermissions {
    /// 允许的 frps 服务器地址白名单（域名或 IP[:端口]）
    #[serde(default)]
    pub allowed_servers: Vec<String>,
    /// 是否允许自定义服务器（false=仅白名单内的服务器）
    #[serde(default)]
    pub allow_custom_server: bool,
}

/// OAuth2 配置（auth.type=oauth2 时必填）
///
/// 参见 FRP_MANAGER_DESIGN.md §6.3。本地启动 HTTP 服务监听 redirectPort 接收回调，
/// 浏览器跳转走 `crate::minecraft::system::shell::open_url`，token 交换在后端完成。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Config {
    /// 授权页 URL
    pub authorize_url: String,
    /// token 交换 URL
    pub token_url: String,
    /// 客户端 ID
    pub client_id: String,
    /// 权限范围
    #[serde(default)]
    pub scopes: Vec<String>,
    /// 回调端口（本地启动 HTTP 服务接收 callback）
    pub redirect_port: u16,
}

/// Device Code 配置（auth.type=device_code 时必填）
///
/// 参见 FRP_MANAGER_DESIGN.md §6.4。POST deviceCodeUrl 获取设备码，
/// 前端显示用户码 + 验证链接 + 倒计时，后端按 interval 轮询 tokenUrl。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeConfig {
    /// 设备码请求 URL
    pub device_code_url: String,
    /// token 轮询 URL
    pub token_url: String,
    /// 客户端 ID
    pub client_id: String,
    /// 权限范围
    #[serde(default)]
    pub scopes: Vec<String>,
    /// 轮询间隔（秒），默认 5
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,
}

/// API Key 配置（auth.type=api_key 时必填）
///
/// 用户手动获取 Key 填入，存储到 OS 密钥存储，调用厂商 API 时注入请求头。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyConfig {
    /// 获取 API Key 的 URL（前端提供跳转入口）
    pub obtain_url: String,
    /// API Key 在请求头中的字段名
    pub header_name: String,
}

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

/// serde 默认值：bundled
fn default_distribution() -> String {
    "bundled".to_string()
}

/// serde 默认值：none
fn default_auth_type() -> String {
    "none".to_string()
}

/// serde 默认值：Device Code 轮询间隔 5 秒
fn default_poll_interval() -> u64 {
    5
}

// ============================================================
// 路径辅助函数
// ============================================================

/// Frp 数据根目录（`<base_dir>/frp/`）
pub fn frp_data_dir() -> PathBuf {
    crate::storage::Storage::instance().base_dir().join("frp")
}

/// 厂商根目录（`<base_dir>/providers/`）
pub fn providers_root() -> PathBuf {
    crate::storage::Storage::instance().base_dir().join("providers")
}

/// 隧道配置文件路径（`<base_dir>/frp/tunnels.json`）
pub fn tunnels_path() -> PathBuf {
    frp_data_dir().join("tunnels.json")
}

/// 厂商启用状态文件（`<base_dir>/frp/providers.json`）
pub fn providers_state_path() -> PathBuf {
    frp_data_dir().join("providers.json")
}

/// 校验厂商 ID 合法性（kebab-case：小写字母 + 数字 + 连字符，
/// 不以连字符开头 / 结尾，最长 64 字符）
pub fn validate_provider_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("厂商 ID 不能为空".to_string());
    }
    if id.len() > 64 {
        return Err("厂商 ID 不能超过 64 字符".to_string());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err("厂商 ID 仅允许小写字母、数字、连字符".to_string());
    }
    if id.starts_with('-') || id.ends_with('-') {
        return Err("厂商 ID 不能以连字符开头或结尾".to_string());
    }
    Ok(())
}

/// frpc 日志目录（`<base_dir>/frp/logs/`）
pub fn frp_logs_dir() -> PathBuf {
    frp_data_dir().join("logs")
}

/// frpc 运行时配置文件目录（`<base_dir>/frp/config/`）
pub fn frp_config_dir() -> PathBuf {
    frp_data_dir().join("config")
}

/// 确保目录存在
pub fn ensure_dir(path: &std::path::Path) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("创建目录失败 {}: {}", path.display(), e))?;
    }
    Ok(())
}
