//! Frp 内网穿透命令模块（编排层）
//!
//! 厂商存放于 `<base_dir>/providers/<provider_id>/`，隧道配置持久化于
//! `<base_dir>/frp/tunnels.json`，frpc 日志写入 `<base_dir>/frp/logs/`。
//! 子模块按职责拆分：provider（列表/状态/启禁）/ install（安装/卸载）/
//! binary（frpc 二进制下载）/ tunnel（CRUD/配置生成）/ process（进程管理/日志）/ sandbox（校验）。
//! 所有子模块函数由 `utils::frp_manager::dispatch` 统一反序列化参数后调用。

pub mod binary;
pub mod install;
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
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            auth_type: default_auth_type(),
        }
    }
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
