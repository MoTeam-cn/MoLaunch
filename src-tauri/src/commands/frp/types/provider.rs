//! 厂商清单与 frpc 二进制分发配置

use serde::{Deserialize, Serialize};

use super::auth::AuthConfig;

/// serde 默认值：bundled
fn default_distribution() -> String {
    "bundled".to_string()
}

/// serde 默认值：进程超时 30 秒
fn default_process_timeout_ms() -> u64 {
    30_000
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
    /// 厂商图标绝对路径（可选，由后端填充，前端用 convertFileSrc 渲染）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
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
    /// 认证交互层文件引用（如 "auth.json"），分离认证 UI 配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_file: Option<String>,
    /// Open API 规范入口（endpointsFile 指向 api/endpoints.json）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiRef>,
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

/// API 规范引用（manifest.api 字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiRef {
    /// endpoints.json 相对路径（如 "api/endpoints.json"）
    pub endpoints_file: String,
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

/// frpc 二进制分发配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryConfig {
    /// 分发方式：bundled=随厂商包打包 / url=按需下载
    #[serde(default = "default_distribution")]
    pub distribution: String,
    /// distribution=bundled 时：厂商自带 frpc 相对路径（单平台时使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// distribution=bundled 时：按平台映射的 frpc 相对路径（多平台时使用）
    ///
    /// key 格式 `{os}_{arch}`，如 `windows_amd64` / `linux_arm64` / `darwin_arm64`。
    /// 优先于 `path` 字段：若当前平台在 paths 中存在则使用 paths 的值，否则回退到 path。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<std::collections::HashMap<String, String>>,
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
