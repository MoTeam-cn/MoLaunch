//! Frp 公共服务接口客户端
//! 对接 api-server `/v1/frp/*`：frpc 二进制分发（manifest）+ 公共 frps 服务器
//! （servers/allocate/release/keepalive）。GET 明文响应携带 JWT；POST 走 ECIES 加密信封
//! 由 `OnlineClient::call_v1` 统一处理。字段命名：apiServer 返回 snake_case，客户端结构体用
//! camelCase + `alias` 同时支持反序列化（snake_case）与序列化给前端（camelCase）。

use serde::{Deserialize, Serialize};

use crate::api_paths;
use super::client::{BusinessResult, ClientError, OnlineClient};
use super::storage::DeviceCredentials;

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

/// 公共 frps 服务器信息
///
/// external 服务器附带 `publicToken`（客户端直接连接，无需 allocate）；
/// self_managed 服务器 `publicToken` 为空，需调 allocate 获取 per-user token + remotePort。
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
    /// 服务器类型：`self_managed` / `external`
    #[serde(alias = "server_type")]
    pub server_type: String,
    /// 仅 external 返回公共 token；self_managed 为空字符串
    #[serde(alias = "public_token")]
    pub public_token: String,
    #[serde(alias = "online_users")]
    pub online_users: i32,
    #[serde(alias = "max_users")]
    pub max_users: i32,
    /// 负载百分比（0-100）
    #[serde(alias = "load_percent")]
    pub load_percent: i32,
    /// 是否可分配
    pub allocatable: bool,
    #[serde(alias = "tls_enabled")]
    pub tls_enabled: bool,
}

// 分配端口（POST /v1/frp/allocate）

/// 分配端口请求（加密信封内明文）
///
/// 字段命名与 apiServer `AllocateRequest` 一致（snake_case），
/// 因为此结构体仅用于构造加密请求体，不参与前端序列化。
#[derive(Debug, Clone, Serialize)]
pub struct AllocateRequest {
    /// 用户选择的服务器 ID（来自 GET /servers）
    pub server_id: String,
    /// 隧道类型（tcp / udp）
    pub tunnel_type: String,
}

/// 分配端口响应（加密信封内明文，解密后由 `call_v1` 反序列化为 `BusinessResult<AllocateResponse>`）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocateResponse {
    /// 分配的服务器信息
    pub server: AllocateServerInfo,
    /// 用户专属远程端口（frpc remote_port）
    #[serde(alias = "remote_port")]
    pub remote_port: i32,
    /// frps 鉴权 token（per-user，frpc token 字段）
    #[serde(alias = "frp_token")]
    pub frp_token: String,
    /// 分配过期时间（Unix 秒，超时未续期则回收）
    #[serde(alias = "expires_at")]
    pub expires_at: i64,
    /// 分配 ID（用于 release / keepalive）
    #[serde(alias = "allocation_id")]
    pub allocation_id: String,
}

/// 分配响应中的服务器信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocateServerInfo {
    pub id: String,
    #[serde(alias = "server_addr")]
    pub server_addr: String,
    #[serde(alias = "server_port")]
    pub server_port: i32,
    #[serde(alias = "tls_enabled")]
    pub tls_enabled: bool,
}

// 释放 / 续期（POST /v1/frp/release|keepalive）

/// 释放分配请求（加密信封内明文）
#[derive(Debug, Clone, Serialize)]
pub struct ReleaseRequest {
    pub allocation_id: String,
}

/// 续期分配请求（加密信封内明文）
#[derive(Debug, Clone, Serialize)]
pub struct KeepaliveRequest {
    pub allocation_id: String,
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

    /// 分配端口 + per-user token（POST /v1/frp/allocate）
    ///
    /// 请求/响应均走 ECIES 加密信封，CSRF 自动获取。
    /// - self_managed 服务器：原子分配端口，返回 per-user token + remotePort
    /// - external 服务器：直接返回公共 token，remotePort=0
    pub async fn frp_allocate(
        &self,
        creds: &DeviceCredentials,
        req: &AllocateRequest,
    ) -> Result<BusinessResult<AllocateResponse>, ClientError> {
        let body = serde_json::to_value(req)?;
        self.call_v1::<AllocateResponse>(creds, "POST", api_paths::FRP_ALLOCATE, Some(&body), true)
            .await
    }

    /// 释放分配（POST /v1/frp/release）
    ///
    /// 用户停止隧道时调用，便于端口回收。即使不调用，过期后也会自动回收。
    pub async fn frp_release(
        &self,
        creds: &DeviceCredentials,
        allocation_id: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let body = serde_json::json!({ "allocation_id": allocation_id });
        self.call_v1::<serde_json::Value>(creds, "POST", api_paths::FRP_RELEASE, Some(&body), true)
            .await
    }

    /// 续期分配（POST /v1/frp/keepalive）
    ///
    /// frpc 运行期间定时调用，延长 `expiresAt`。续期失败提示用户重新分配。
    pub async fn frp_keepalive(
        &self,
        creds: &DeviceCredentials,
        allocation_id: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let body = serde_json::json!({ "allocation_id": allocation_id });
        self.call_v1::<serde_json::Value>(creds, "POST", api_paths::FRP_KEEPALIVE, Some(&body), true)
            .await
    }
}
