//! 厂商 Open API 引擎：endpoints.json 解析 + API 调用 + frpc 配置生成
//! 子模块：registry（规格加载）/ executor（请求执行）/ config_gen / envelope / http / jsonpath

mod executor;
mod registry;

pub mod config_gen;
pub mod envelope;
pub mod http;
pub mod jsonpath;

pub use executor::fetch_tunnels;
pub use registry::load_api_spec;

use serde::Serialize;

// 统一隧道数据（API 响应映射后的标准格式）
/// 隧道信息（从厂商 API 响应映射后的统一格式）
///
/// 对应 endpoints.json 中 tunnelFields 定义的字段。
/// fields 模式下启动器按这些字段拼装 frpc 配置。
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInfo {
    pub id: String,
    pub name: String,
    pub tunnel_type: String,
    pub status: String,
    pub server_host: String,
    pub server_port: String,
    pub token: String,
    pub local_host: String,
    pub local_port: String,
    pub remote_port: String,
    pub custom_domain: String,
}

/// 账号信息（从厂商 API 响应映射）
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub token: String,
}