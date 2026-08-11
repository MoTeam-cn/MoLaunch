//! Frp 公共服务接口客户端
//! 对接 api-server `/v1/frp/*`：frpc 二进制分发（manifest）+ 公共 frps 服务器列表
//! （servers）。GET 明文响应携带 JWT。字段命名：apiServer 返回 snake_case，客户端结构体用
//! camelCase + `alias` 同时支持反序列化（snake_case）与序列化给前端（camelCase）。

use serde::{Deserialize, Serialize};

use super::client::{BusinessResult, ClientError, OnlineClient};
use super::storage::DeviceCredentials;
use crate::api_paths;

// frpc manifest（GET /v1/frp/manifest）

/// frpc 版本清单查询参数
///
/// 字段命名与 apiServer `FrpManifestQuery` 一致（snake_case），
/// 因为此结构体仅用于构造 query string，不参与前端序列化。
#[derive(Debug, Clone, Serialize)]
pub struct FrpManifestQuery {
    /// 组件类型：`client`（frpc）| `server`（frps）
    pub component: String,
    /// 目标平台：`windows` | `macos` | `linux`
    pub target: String,
    /// 架构：`x86_64` | `aarch64` | `i686` | `armv7`
    pub arch: String,
    /// 客户端当前 frp 版本号（如 `0.60.0`）
    pub current_version: String,
}

/// frpc 版本清单响应
///
/// `data=None` 表示已是最新版本（apiServer 返回 `code=1, msg="已是最新版本", data=null`）。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrpManifest {
    /// 最新版本号
    #[serde(alias = "version")]
    pub version: String,
    /// 发布时间（RFC 3339，UTC）
    #[serde(alias = "pub_date")]
    pub pub_date: String,
    /// 压缩包下载 URL（S3 启用时为 presigned URL）
    #[serde(alias = "url")]
    pub url: String,
    /// Ed25519 签名 base64（frp 官方无签名，留空）
    #[serde(alias = "signature")]
    pub signature: String,
    /// 更新日志（Markdown）
    #[serde(alias = "notes")]
    pub notes: String,
    /// GitHub Release 页面 URL
    #[serde(alias = "release_url")]
    pub release_url: String,
    /// 当前灰度比例（0-100，客户端仅展示用）
    #[serde(alias = "rollout_pct")]
    pub rollout_pct: u32,
}

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
    /// 查询 frpc 更新（GET /v1/frp/manifest）
    ///
    /// 返回 `BusinessResult<FrpManifest>`：
    /// - `code=1, data=Some(manifest)`：有可用更新
    /// - `code=1, data=None, msg="已是最新版本"`：已是最新版本
    /// - 其他 code：业务错误（如 503 服务未启用）
    pub async fn frp_get_manifest(
        &self,
        creds: &DeviceCredentials,
        query: &FrpManifestQuery,
    ) -> Result<BusinessResult<FrpManifest>, ClientError> {
        let encoded_component = urlencoding::encode(&query.component);
        let encoded_target = urlencoding::encode(&query.target);
        let encoded_arch = urlencoding::encode(&query.arch);
        let encoded_version = urlencoding::encode(&query.current_version);
        let path = api_paths::FRP_MANIFEST
            .replace("{component}", encoded_component.as_ref())
            .replace("{target}", encoded_target.as_ref())
            .replace("{arch}", encoded_arch.as_ref())
            .replace("{current_version}", encoded_version.as_ref());
        self.call_v1::<FrpManifest>(creds, "GET", &path, None, false)
            .await
    }

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
