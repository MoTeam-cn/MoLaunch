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
    /// 配置文件要求（无 config 端点时用于手工生成；有 config 端点时仅作说明）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_requirements: Option<ConfigRequirements>,
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

/// 厂商配置文件要求。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRequirements {
    #[serde(default)]
    pub fields: Vec<ConfigRequiredField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRequiredField {
    pub path: String,
    pub source: ConfigFieldSource,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFieldSource {
    pub endpoint: String,
    pub field: String,
    pub unit: String,
    pub target_unit: String,
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
    /// frpc 版本号（manifest 中注明，如 "0.51.3"）
    ///
    /// 作为 frpc 更新的唯一判断依据：无论 bundled 还是 url，
    /// 启动器将该版本写入厂商目录 `frpc_version.txt`，后续
    /// `ensure_frpc` 比对「manifest.frpc_version vs 记录值」，
    /// 不一致才执行更新（重新下载/替换 frpc），一致则保持不动。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frpc_version: Option<String>,
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
    /// frpc 启动方式声明（通用机制，适配厂商魔改 frpc）
    ///
    /// `mode=config`（默认）：`<frpc> -c <config.toml>` 启动，走启动器生成的配置文件。
    /// `mode=command`：厂商 frpc 不接受标准配置文件，改用命令参数直连
    /// （如 Lolia 的 `-t <tunnelId>:<token>`）。此时 `command` 为命令行模板，
    /// 支持占位符：
    /// - `{frpc}`：frpc 二进制绝对路径
    /// - `{tunnelId}`：远程隧道自增 ID（厂商隧道列表的 id，如 Lolia 的 `16977`）
    /// - `{token}`：隧道鉴权 token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch: Option<LaunchConfig>,
}

/// frpc 启动方式配置（binary.launch 字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchConfig {
    /// 启动模式：config（默认，-c 配置文件）/ command（命令模板直连）
    #[serde(default = "default_launch_mode")]
    pub mode: String,
    /// mode=command 时的命令行模板（不含 frpc 路径，用 {frpc} 占位，
    /// 需在各参数前补占位符，如 `{frpc} -t {tunnelName}:{token}`）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// 默认启动模式：config
fn default_launch_mode() -> String {
    "config".to_string()
}

/// URL 下载配置（distribution=url 时使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadConfig {
    /// 默认下载 URL（回退）
    pub url: String,
    /// 按平台映射的下载 URL（key 格式 `{os}_{arch}`，如 `windows_amd64`）
    ///
    /// 优先于 `url`：若当前平台在 urls 中存在则使用 urls 的值，否则回退到 url。
    /// 适配"同一厂商不同架构 frpc 用不同下载地址"的场景（如 GitHub Releases 按平台分发的二进制）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urls: Option<std::collections::HashMap<String, String>>,
    /// 按平台映射的下载目标相对路径（key 格式同 urls）
    ///
    /// 若当前平台在 target_paths 中存在则优先使用，否则回退到 target_path。
    /// 适配"不同平台文件名不同（如 .exe / 无后缀）"的场景。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_paths: Option<std::collections::HashMap<String, String>>,
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
