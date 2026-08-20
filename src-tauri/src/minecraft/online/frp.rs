//! Frp 公共服务接口客户端
//! 对接 api-server `/v1/frp/*`：公共 frps 服务器列表（servers）。
//! GET 明文响应携带 JWT。字段命名：apiServer 返回 snake_case，客户端结构体用
//! camelCase + `alias` 同时支持反序列化（snake_case）与序列化给前端（camelCase）。

use serde::{Deserialize, Serialize};

use super::client::{BusinessResult, ClientError, OnlineClient};
use super::storage::DeviceCredentials;
use crate::api_paths;

// 公共 frps 服务器（GET /v1/frp/servers）

/// 公共 frps 服务器信息（GET /v1/frp/servers）
///
/// apiServer 直接返回完整连接信息（公共 token + 地址端口），客户端无需再调用分配接口。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicFrpServer {
    pub id: String,
    pub name: String,
    pub region: String,
    #[serde(alias = "server_addr")]
    pub server_addr: String,
    #[serde(alias = "server_port")]
    pub server_port: i32,
    /// 公共共享 token（frpc token 字段）
    #[serde(alias = "public_token")]
    pub public_token: String,
    #[serde(alias = "tls_enabled")]
    pub tls_enabled: bool,
}

// OnlineClient 扩展方法

impl OnlineClient {
    /// 列出可用的公共 frps 服务器（GET /v1/frp/servers）
    ///
    /// 明文响应，自动携带 JWT。返回所有 `status=active` 且心跳未超时的服务器。
    pub async fn frp_list_public_servers(
        &self,
        creds: &DeviceCredentials,
    ) -> Result<BusinessResult<Vec<PublicFrpServer>>, ClientError> {
        self.call_v1::<Vec<PublicFrpServer>>(creds, "GET", api_paths::FRP_SERVERS, None, false)
            .await
    }
}
