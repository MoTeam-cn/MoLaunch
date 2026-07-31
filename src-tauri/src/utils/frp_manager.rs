//! Frp 模块统一分发逻辑（frp_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，覆盖：
//! - 厂商管理：list_providers / enable_provider / disable_provider（provider）；
//!   ensure_frpc（binary）；install_provider_from_dir / install_provider_from_zip /
//!   uninstall_provider（install）
//! - 隧道管理：list_tunnels / create_tunnel / delete_tunnel
//! - frpc 进程：start_tunnel / stop_tunnel / get_tunnel_status
//! - 日志管理：list_log_files / read_log_file
//! - 公共 frps 服务器：list_public_servers / allocate_public_server /
//!   release_public_server / keepalive_public_server（对接 apiServer `/v1/frp/*`）
//!
//! start_tunnel 需要 `AppHandle` 用于推送 frpc-log / frp-tunnel-status event。
//! ensure_frpc 与公共服务器 action 需要 `AppState` 用于读取 apiServer 配置 + 加载设备凭证。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::commands::frp;
use crate::commands::frp::tunnel::{CreateTunnelParams, TunnelIdParams, UpdateTunnelParams};
use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::frp::{
    AllocateRequest, AllocateResponse, PublicFrpServer,
};
use crate::minecraft::online::storage::DeviceCredentials;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// 参数结构体
// ============================================================

/// ensure_frpc 参数（provider_id 可选，默认系统默认厂商）
#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnsureFrpcParams {
    #[serde(default)]
    pub provider_id: Option<String>,
}

/// 安装厂商参数（source_dir 可为文件夹路径或 ZIP 路径）
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProviderParams {
    pub source_dir: String,
}

/// 厂商 ID 参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIdParams {
    pub provider_id: String,
}

/// 读取日志参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadLogParams {
    pub tunnel_id: String,
    #[serde(default)]
    pub max_lines: Option<usize>,
}

/// 分配公共服务器端口参数（对应 apiServer `AllocateRequest`）
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocatePublicServerParams {
    pub server_id: String,
    pub tunnel_type: String,
}

/// 释放/续期分配参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationIdParams {
    pub allocation_id: String,
}

/// 保存 API Key 参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveApiKeyParams {
    pub provider_id: String,
    pub api_key: String,
}

/// 执行厂商认证适配器脚本参数（对应 §7.5 沙箱）
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunAuthAdapterParams {
    pub provider_id: String,
    /// 要执行的命令（必须在厂商 allowedCommands 白名单内）
    pub command: String,
    /// 命令参数
    #[serde(default)]
    pub args: Vec<String>,
}

// ============================================================
// 在线调用辅助函数（与 signaling_manager 风格一致）
// ============================================================

/// 加载设备凭证（需已注册），access token 过期时自动 refresh
async fn load_creds(state: &AppState) -> Result<DeviceCredentials, String> {
    crate::utils::online_manager::load_creds_with_auto_refresh(state).await
}

/// 创建 OnlineClient（读取配置中的 api_server_url）
async fn make_client(state: &AppState) -> OnlineClient {
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    OnlineClient::new(&base_url)
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // ============================================================
    // 厂商管理
    // ============================================================

    d.register("list_providers", handler!(state, _app, _params, {
        let r = frp::provider::list_providers(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("ensure_frpc", handler!(state, _app, params, {
        // 兼容空 params（{} 或 null）：unwrap_or_default 返回 provider_id=None
        let p: EnsureFrpcParams = serde_json::from_value(params).unwrap_or_default();
        let r = frp::binary::ensure_frpc(&state, p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("install_provider_from_dir", handler!(_state, _app, params, {
        let p: InstallProviderParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::install::install_provider_from_dir(p.source_dir).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("install_provider_from_zip", handler!(_state, _app, params, {
        let p: InstallProviderParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::install::install_provider_from_zip(p.source_dir).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("uninstall_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::install::uninstall_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("enable_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::provider::enable_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("disable_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::provider::disable_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // ============================================================
    // 隧道管理
    // ============================================================

    d.register("list_tunnels", handler!(_state, _app, _params, {
        let r = frp::process::list_tunnels_with_status().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("create_tunnel", handler!(_state, _app, params, {
        let p: CreateTunnelParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::sandbox::validate_tunnel(&p)?;
        let r = frp::tunnel::create_tunnel(p).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("delete_tunnel", handler!(_state, _app, params, {
        let p: TunnelIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        // 先停止运行中的进程（忽略错误，可能未在运行）
        let _ = frp::process::stop_tunnel(p.id.clone()).await;
        frp::tunnel::delete_tunnel(p.id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("update_tunnel", handler!(_state, _app, params, {
        let p: UpdateTunnelParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::sandbox::validate_tunnel_update(&p)?;
        let r = frp::tunnel::update_tunnel(p).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // ============================================================
    // frpc 进程管理
    // ============================================================

    d.register("start_tunnel", handler!(state, app, params, {
        let p: TunnelIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::process::start_tunnel(&state, p.id, app.clone()).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("stop_tunnel", handler!(_state, _app, params, {
        let p: TunnelIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::process::stop_tunnel(p.id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("get_tunnel_status", handler!(_state, _app, params, {
        let p: TunnelIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::process::get_tunnel_status(p.id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // ============================================================
    // 日志管理
    // ============================================================

    d.register("list_log_files", handler!(_state, _app, _params, {
        let r = frp::process::list_log_files().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("read_log_file", handler!(_state, _app, params, {
        let p: ReadLogParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::process::read_log_file(p.tunnel_id, p.max_lines).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // ============================================================
    // 认证体系（阶段三：OAuth2 / Device Code / API Key）
    // ============================================================

    d.register("get_auth_status", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::auth::get_auth_status(&p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("start_oauth2", handler!(state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::auth::start_oauth2(&state, &p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("start_device_code", handler!(state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::auth::start_device_code(&state, &p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("poll_device_code", handler!(state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::auth::poll_device_code(&state, &p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("refresh_token", handler!(state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::auth::refresh_token(&state, &p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("revoke_auth", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::auth::revoke_auth(&p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("save_api_key", handler!(_state, _app, params, {
        let p: SaveApiKeyParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::auth::save_api_key(&p.provider_id, &p.api_key).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // ============================================================
    // 厂商 API 引擎（阶段三：api-schema.json 解析 + 配置拉取）
    // ============================================================

    d.register("fetch_vendor_config", handler!(state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::api_schema::fetch_vendor_config(&state, &p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // ============================================================
    // 认证适配器脚本沙箱（阶段四 §7.5）
    // ============================================================

    d.register("run_auth_adapter", handler!(_state, _app, params, {
        let p: RunAuthAdapterParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::sandbox::run_auth_adapter(&p.provider_id, p.command, p.args).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // ============================================================
    // 公共 frps 服务器（对接 apiServer `/v1/frp/*`）
    // ============================================================

    // 列出可用的公共 frps 服务器（GET /v1/frp/servers）
    //
    // 明文响应，自动携带 JWT。前端据此展示服务器列表供用户选择。
    d.register("list_public_servers", handler!(state, _app, _params, {
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Frp] list_public_servers");
        let result = client.frp_list_public_servers(&creds).await
            .map_err(|e| {
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
    }));

    // 分配端口 + per-user token（POST /v1/frp/allocate）
    //
    // 请求/响应均走 ECIES 加密信封。self_managed 服务器原子分配端口，
    // external 服务器直接返回公共 token（remotePort=0）。
    d.register("allocate_public_server", handler!(state, _app, params, {
        let p: AllocatePublicServerParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Frp] allocate_public_server: server_id={}, tunnel_type={}",
            p.server_id, p.tunnel_type
        );
        let req = AllocateRequest {
            server_id: p.server_id,
            tunnel_type: p.tunnel_type,
        };
        let result = client.frp_allocate(&creds, &req).await
            .map_err(|e| {
                log_error!("[Frp] allocate_public_server 失败: {}", e);
                e.to_string()
            })?;
        if result.code != 1 {
            return Err(format!(
                "分配公共服务器失败 [code={}]: {}",
                result.code, result.msg
            ));
        }
        let resp: AllocateResponse = result.data.ok_or_else(|| {
            "apiServer 未返回分配结果".to_string()
        })?;
        log_info!(
            "[Frp] 分配成功: allocation_id={}, remote_port={}",
            resp.allocation_id, resp.remote_port
        );
        serde_json::to_value(resp).map_err(|e| e.to_string())
    }));

    // 释放分配（POST /v1/frp/release）
    //
    // 用户停止隧道时调用，便于端口回收。即使不调用，过期后也会自动回收。
    d.register("release_public_server", handler!(state, _app, params, {
        let p: AllocationIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Frp] release_public_server: allocation_id={}", p.allocation_id);
        let result = client.frp_release(&creds, &p.allocation_id).await
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
    }));

    // 续期分配（POST /v1/frp/keepalive）
    //
    // frpc 运行期间定时调用，延长 expiresAt。续期失败提示用户重新分配。
    d.register("keepalive_public_server", handler!(state, _app, params, {
        let p: AllocationIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Frp] keepalive_public_server: allocation_id={}", p.allocation_id);
        let result = client.frp_keepalive(&creds, &p.allocation_id).await
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
    }));

    d
});

/// Frp 管理 action 分发入口
pub async fn dispatch(
    _state: AppState,
    _app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(_state, _app, req).await
}
