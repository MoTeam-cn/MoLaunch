//! 公共 frps 服务器 action 注册（对接 apiServer `/v1/frp/*`）。
//!
//! 包含：列出服务器、分配端口 + per-user token、释放分配、续期分配。
//! 请求/响应走 ECIES 加密信封（self_managed 服务器原子分配端口）。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::frp::{AllocateRequest, AllocateResponse, PublicFrpServer};
use crate::minecraft::online::storage::DeviceCredentials;
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;

use super::{AllocatePublicServerParams, AllocationIdParams};

/// 加载设备凭证（需已注册），access token 过期时自动 refresh
async fn load_creds(state: &AppState) -> Result<DeviceCredentials, String> {
    crate::commands::online::manager::load_creds_with_auto_refresh(state).await
}

/// 创建 OnlineClient（读取配置中的 api_server_url）
async fn make_client(state: &AppState) -> OnlineClient {
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    OnlineClient::new(&base_url)
}

/// 注册公共 frps 服务器相关 action
pub fn register(d: &mut Dispatcher) {
    // 列出可用的公共 frps 服务器（GET /v1/frp/servers）
    //
    // 明文响应，自动携带 JWT。前端据此展示服务器列表供用户选择。
    d.register(
        "list_public_servers",
        handler!(state, _app, _params, {
            let creds = load_creds(&state).await?;
            let client = make_client(&state).await;
            log_debug!("[Frp] list_public_servers");
            let result = client.frp_list_public_servers(&creds).await.map_err(|e| {
                log_error!("[Frp] list_public_servers 失败: {}", e);
                e.to_string()
            })?;
            if result.code != 1 {
                return Err(format!(
                    "列出公共服务器失败 [code={}]: {}",
                    result.code, result.msg
                ));
            }
            let servers: Vec<PublicFrpServer> = result.data.unwrap_or_default();
            serde_json::to_value(servers).map_err(|e| e.to_string())
        }),
    );

    // 分配端口 + per-user token（POST /v1/frp/allocate）
    //
    // 请求/响应均走 ECIES 加密信封。self_managed 服务器原子分配端口，
    // external 服务器直接返回公共 token（remotePort=0）。
    d.register(
        "allocate_public_server",
        handler!(state, _app, params, {
            let p: AllocatePublicServerParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = load_creds(&state).await?;
            let client = make_client(&state).await;
            log_info!(
                "[Frp] allocate_public_server: server_id={}, tunnel_type={}",
                p.server_id,
                p.tunnel_type
            );
            let req = AllocateRequest {
                server_id: p.server_id,
                tunnel_type: p.tunnel_type,
            };
            let result = client.frp_allocate(&creds, &req).await.map_err(|e| {
                log_error!("[Frp] allocate_public_server 失败: {}", e);
                e.to_string()
            })?;
            if result.code != 1 {
                return Err(format!(
                    "分配公共服务器失败 [code={}]: {}",
                    result.code, result.msg
                ));
            }
            let resp: AllocateResponse = result
                .data
                .ok_or_else(|| "apiServer 未返回分配结果".to_string())?;
            log_info!(
                "[Frp] 分配成功: allocation_id={}, remote_port={}",
                resp.allocation_id,
                resp.remote_port
            );
            serde_json::to_value(resp).map_err(|e| e.to_string())
        }),
    );

    // 释放分配（POST /v1/frp/release）
    //
    // 用户停止隧道时调用，便于端口回收。即使不调用，过期后也会自动回收。
    d.register(
        "release_public_server",
        handler!(state, _app, params, {
            let p: AllocationIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = load_creds(&state).await?;
            let client = make_client(&state).await;
            log_info!(
                "[Frp] release_public_server: allocation_id={}",
                p.allocation_id
            );
            let result = client
                .frp_release(&creds, &p.allocation_id)
                .await
                .map_err(|e| {
                    log_error!("[Frp] release_public_server 失败: {}", e);
                    e.to_string()
                })?;
            if result.code != 1 {
                return Err(format!(
                    "释放分配失败 [code={}]: {}",
                    result.code, result.msg
                ));
            }
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    // 续期分配（POST /v1/frp/keepalive）
    //
    // frpc 运行期间定时调用，延长 expiresAt。续期失败提示用户重新分配。
    d.register(
        "keepalive_public_server",
        handler!(state, _app, params, {
            let p: AllocationIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = load_creds(&state).await?;
            let client = make_client(&state).await;
            log_debug!(
                "[Frp] keepalive_public_server: allocation_id={}",
                p.allocation_id
            );
            let result = client
                .frp_keepalive(&creds, &p.allocation_id)
                .await
                .map_err(|e| {
                    log_error!("[Frp] keepalive_public_server 失败: {}", e);
                    e.to_string()
                })?;
            if result.code != 1 {
                return Err(format!(
                    "续期分配失败 [code={}]: {}",
                    result.code, result.msg
                ));
            }
            serde_json::to_value(result.data).map_err(|e| e.to_string())
        }),
    );
}
