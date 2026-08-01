//! Frp 共享数据类型：隧道、厂商清单、认证配置、日志文件信息
//!
//! 这些类型在 `frp` 模块及其子模块（provider/install/binary/process/auth/...）间共享，
//! 集中在此处避免循环依赖。`serde` 默认值函数与类型定义放在一起便于维护。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 隧道相关类型
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

// ============================================================
// 厂商信息与清单
// ============================================================

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

// ============================================================
// 认证配置
// ============================================================

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
    /// token 交换 URL（兼容旧版 manifest，新设计改由 endpoints.json authFlows.oauth2.token.url 提供）
    pub token_url: String,
    /// 客户端 ID
    pub client_id: String,
    /// 客户端密钥（可选，部分厂商需要）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
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
    /// 客户端密钥（可选，部分厂商需要）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
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

// ============================================================
// 日志文件信息
// ============================================================

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

// ============================================================
// serde 默认值函数
// ============================================================

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
// Open API 接口规范类型（api/endpoints.json 反序列化结构）
//
// 设计参考：docs/Frp Test/frp/api/endpoints.json
// 厂商接口响应结构各不相同，通过此规范将差异全部做成可配置项。
// ============================================================

/// endpoints.json 顶层结构
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSpec {
    /// 规范版本
    #[serde(default)]
    pub spec_version: Option<String>,
    /// API 基础 URL（如 "https://api.openfrp.net"）
    pub base_url: String,
    /// token 注入配置
    #[serde(default)]
    pub auth: Option<AuthHeader>,
    /// 认证流程定义（请求参数 + 响应解析）
    #[serde(default)]
    pub auth_flows: Option<AuthFlows>,
    /// 全局响应包裹解析（各接口可用 endpoints.*.envelope 覆盖）
    #[serde(default)]
    pub envelope: Option<Envelope>,
    /// 配置获取模式
    pub config: ConfigMode,
    /// API 端点定义
    #[serde(default)]
    pub endpoints: Option<EndpointsDef>,
}

/// token 注入配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthHeader {
    /// 请求头字段名（如 "Authorization"）
    pub header_name: String,
    /// 值前缀（如 "Bearer "，无空格时留空）
    #[serde(default)]
    pub header_prefix: String,
    /// 登录响应中服务器密钥所在响应头名（解密用，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_key_name: Option<String>,
}

/// 认证流程定义
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthFlows {
    /// OAuth2 授权码流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<OAuth2Flow>,
    /// Device Code 设备码流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<DeviceCodeFlow>,
    /// API Key 直传流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyFlow>,
    /// 远程登录流程（如 OpenFRP argoAccess）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_login: Option<RemoteLoginFlow>,
}

/// OAuth2 流程（token 交换 + refresh）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Flow {
    pub token: FlowRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<FlowRequest>,
}

/// Device Code 流程（request + poll）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeFlow {
    pub request: FlowRequest,
    pub poll: FlowRequest,
}

/// API Key 流程（请求头/查询直传）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyFlow {
    pub header_name: String,
    /// 值模板（如 "OPENFRP{apiKey}"）
    pub header_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_name: Option<String>,
}

/// 远程登录流程（request + poll）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteLoginFlow {
    pub request: FlowRequest,
    pub poll: FlowRequest,
}

/// 流程请求定义（通用：method + url + body + response 解析）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: serde_json::Value,
    /// 响应字段提取规则
    #[serde(default)]
    pub response: HashMap<String, FieldExtractor>,
    /// Device Code 轮询时的 pending 错误标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_error: Option<String>,
}

/// 字段提取器（从 body 或 header 按 path/name 取值）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldExtractor {
    /// 来源：body / header
    pub from: String,
    /// from=body 时为 JSONPath（如 "$.access_token"）；from=header 时为头字段名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 响应包裹解析
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    /// 成功判断字段路径（JSONPath，如 "$.flag"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_field: Option<String>,
    /// 该字段等于此值时视为成功
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_value: Option<serde_json::Value>,
    /// 失败时错误消息字段路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_field: Option<String>,
    /// 数据字段路径（接口未覆盖时兜底）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_field: Option<String>,
}

/// 配置获取模式
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMode {
    /// url=厂商接口直返配置 / fields=启动器按字段拼装 / args=frpc 以启动参数运行
    pub mode: String,
    /// 配置格式（ini/toml），默认 ini
    #[serde(default = "default_config_format")]
    pub format: String,
    /// mode=args 时的参数模板（如 ["-u", "{token}", "-p", "{ids}"]）
    #[serde(default)]
    pub args: Vec<String>,
}

/// serde 默认值：配置格式 ini
fn default_config_format() -> String {
    "ini".to_string()
}

/// API 端点定义
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EndpointsDef {
    /// 账号信息端点（可选，验证授权有效性）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<EndpointDef>,
    /// 隧道端点集合
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnels: Option<TunnelsDef>,
}

/// 单个端点定义
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointDef {
    pub method: String,
    pub path: String,
    #[serde(default = "default_auth_type_bearer")]
    pub auth_type: String,
    #[serde(default)]
    pub query: HashMap<String, String>,
    /// 路径参数映射（如 { "tunnelId": "id" } 表示 {tunnelId} 替换为隧道 id 字段）
    #[serde(default)]
    pub path_params: HashMap<String, String>,
    /// 接口级 envelope 覆盖
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Envelope>,
    /// 响应映射
    pub response: ResponseDef,
}

/// 隧道端点集合
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TunnelsDef {
    /// 隧道列表端点（必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<EndpointDef>,
    /// 隧道详情端点（可选，列表未返回完整字段时启用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<EndpointDef>,
    /// 配置获取端点（可选，config.mode=url 时使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<EndpointDef>,
}

/// 响应映射定义
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseDef {
    /// 列表位置（仅 tunnels.list，如 "$.data[*].proxies[*]"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_field: Option<String>,
    /// 隧道 ID 字段名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_id_field: Option<String>,
    /// 隧道名称字段名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_name_field: Option<String>,
    /// 数据字段路径（如 "$.data"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_field: Option<String>,
    /// 字段映射（key=统一字段名，value=厂商字段名或 {field, split} 对象）
    #[serde(default)]
    pub fields: HashMap<String, FieldMapping>,
    /// 响应编码（text/json），config 端点用 text 取原始字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

/// 字段映射（字符串=字段名，对象={field, split} 从合并字段拆分）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldMapping {
    /// 厂商字段名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// 从合并字段拆分的分隔符（如 ":" 从 "host:port" 拆分）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<String>,
    /// 直接字符串值（如 "{account.token}" 取账号信息 token）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// serde 默认值：bearer
fn default_auth_type_bearer() -> String {
    "bearer".to_string()
}

// ============================================================
// auth.json 认证交互层类型
// ============================================================

/// auth.json 结构（认证交互层配置）
///
/// 仅描述用户交互方式（授权页 URL、回调端口等），
/// 实际网络请求与响应解析见 endpoints.json 的 authFlows。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFile {
    /// 认证类型：none / oauth2 / device_code / api_key
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<AuthFileOAuth2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<AuthFileDeviceCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<AuthFileApiKey>,
}

/// auth.json 中的 OAuth2 交互配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFileOAuth2 {
    pub authorize_url: String,
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub redirect_port: u16,
}

/// auth.json 中的 Device Code 交互配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFileDeviceCode {
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,
}

/// auth.json 中的 API Key 交互配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFileApiKey {
    pub obtain_url: String,
    pub header_name: String,
}
