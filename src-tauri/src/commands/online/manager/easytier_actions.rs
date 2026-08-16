//! easytier / Scaffolding 联机 IPC 动作。
//!
//! - `easytier_join`：拉起 easytier-core 加入虚拟网络（房主固定 IP，房客 DHCP）
//! - `easytier_stop`：停止当前 easytier 子进程
//! - `scaffolding_host_start`：房主一站式启动（探测 MC 端口 → 联机中心 → easytier）
//! - `scaffolding_host_stop`：停止联机中心与 easytier
//! - `scaffolding_client_probe`：房客解析房间码 → 加入网络 → 探测房主 MC 服务

use serde::{Deserialize, Serialize};

use crate::handler;
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::scaffolding::client as scaffolding_client;
use crate::minecraft::online::scaffolding::code as room_code;
use crate::minecraft::online::scaffolding::easytier::{EasyTier, HOST_VIRTUAL_IP};
use crate::minecraft::online::scaffolding::server::{
    ScaffoldingServer, ScaffoldingServerState, DEFAULT_CENTER_PORT,
};
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;
use std::path::PathBuf;
use tauri::{Emitter, Manager};

/// 房客侧默认 hostname（未配置 network_identity 时使用）
const DEFAULT_GUEST_HOSTNAME: &str = "mo-launch-guest";

/// easytier 运行状态推送事件（前端经 useTauriEvent 监听）
const EASYTIER_STATUS_EVENT: &str = "easytier-status";

/// 构造 easytier 运行状态 payload（`easytier-status` 事件推送 / `easytier_status` IPC 查询共用）
fn easytier_status_payload(easytier: &Option<EasyTier>) -> serde_json::Value {
    match easytier {
        Some(e) => serde_json::json!({
            "joined": true,
            "version": e.version(),
            "pid": e.pid(),
            "rpcPortal": e.rpc_portal(),
            "networkName": e.network_name(),
            "virtualIp": e.virtual_ip().unwrap_or(""),
        }),
        None => serde_json::json!({
            "joined": false,
            "version": "",
            "pid": null,
            "rpcPortal": "",
            "networkName": "",
            "virtualIp": "",
        }),
    }
}

/// 构造并推送 easytier 运行状态事件（加入/停止后调用，供设备页展示）
async fn emit_easytier_status(app: &tauri::AppHandle, state: &AppState) {
    let guard = state.easytier.lock().await;
    let payload = easytier_status_payload(&guard);
    let _ = app.emit(EASYTIER_STATUS_EVENT, payload);
}

/// 解析 easytier-core 可执行文件路径。
///
/// 优先级：配置的绝对路径（直接校验存在性）→ 相对路径兼容旧配置（依次尝试
/// resource_dir / exe 同目录）→ 兜底释放内置嵌入式资源到 AppData/.Molaunch/easytier/。
fn resolve_core_path(app: &tauri::AppHandle, configured: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(configured);
    if p.is_absolute() {
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "easytier-core 不存在: {}（请在联机设置中指定正确路径）",
            p.display()
        ));
    }
    // 相对路径（旧配置兼容，如 `sidecar/easytier-core.exe`）
    if !configured.is_empty() {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(dir) = app.path().resource_dir() {
            candidates.push(dir.join(&p));
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join(&p));
            }
        }
        for c in &candidates {
            if c.is_file() {
                return Ok(c.clone());
            }
        }
    }
    // 兜底：释放内置嵌入式资源（默认配置为空时走此路径）
    crate::resources::extract_easytier_core()
        .map_err(|e| format!("释放内置 easytier-core 失败: {e}"))
}

/// 读取配置中的 easytier-core 路径
async fn configured_core_path(state: &AppState) -> String {
    state.config.lock().await.online.easytier_core_path.clone()
}

/// 读取配置中的虚拟网络节点标识（房客 hostname 用）
async fn configured_network_identity(state: &AppState) -> String {
    state.config.lock().await.online.network_identity.clone()
}

/// 按当前游戏进程 PID 探测 MC 局域网端口（沿用 lan_probe 逻辑）
async fn probe_mc_port(state: &AppState) -> Option<u16> {
    let pid = *state.current_pid.lock().await;
    match pid {
        Some(pid) => crate::minecraft::launch::watcher::ports::listening_tcp_ports(pid)
            .into_iter()
            .next(),
        None => None,
    }
}

/// `easytier_join` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EasytierJoinParams {
    /// 虚拟网络名（房主与房客需一致，来自房间码 N 段）
    pub network_name: String,
    /// 虚拟网络密钥（来自房间码 S 段）
    pub network_secret: String,
    /// 是否为房主（房主固定虚拟 IP `10.144.144.1`，房客走 `--dhcp`）
    #[serde(default)]
    pub is_host: bool,
    /// 节点 hostname（房主必须为 `scaffolding-mc-server-{center_port}`，联机中心端口）
    #[serde(default)]
    pub hostname: Option<String>,
    /// 追加 easytier-core CLI 参数（如 `--peers` 公共服务器）
    #[serde(default)]
    pub extra: Option<Vec<String>>,
}

/// `scaffolding_host_start` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingHostStartParams {
    /// 完整房间码 `U/NNNN-NNNN-SSSS-SSSS`（解析网络名与密钥）
    pub room_code: String,
    /// 房主 MC 局域网端口（缺省时按游戏进程探测）
    #[serde(default)]
    pub mc_port: Option<u16>,
}

/// `scaffolding_client_probe` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingClientProbeParams {
    /// 完整房间码 `U/NNNN-NNNN-SSSS-SSSS`
    pub room_code: String,
    /// 联机中心虚拟 IP（缺省取房主固定 `10.144.144.1`）
    #[serde(default)]
    pub center_ip: Option<String>,
    /// 联机中心 TCP 端口（缺省 13448）
    #[serde(default)]
    pub center_port: Option<u16>,
}

/// `easytier_join` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EasytierJoinResponse {
    pub success: bool,
    /// rpc-portal 地址（`127.0.0.1:动态端口`，供后续 easytier CLI 查询）
    pub rpc_portal: String,
    /// 子进程 PID
    pub pid: Option<u32>,
}

/// `scaffolding_host_start` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingHostStartResponse {
    pub success: bool,
    /// 联机中心实际监听端口
    pub center_port: u16,
    /// 中心 hostname（`scaffolding-mc-server-{center_port}`）
    pub hostname: String,
    /// 房主 MC 局域网端口
    pub mc_port: u16,
    /// rpc-portal 地址
    pub rpc_portal: String,
    /// easytier 子进程 PID
    pub pid: Option<u32>,
}

/// `scaffolding_client_probe` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingClientProbeResponse {
    pub success: bool,
    /// 房主虚拟 IP（MC 客户端连接目标，配合 lan_fake 转发）
    pub mc_ip: String,
    /// 房主 MC 局域网端口
    pub mc_port: u16,
}

/// 注册 easytier / Scaffolding 动作到 dispatcher
pub fn register(d: &mut Dispatcher) {
    d.register(
        "easytier_join",
        handler!(state, app, params, {
            let p: EasytierJoinParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;

            // 若已有实例先停止（防泄漏）
            let old = state.easytier.lock().await.take();
            if let Some(old) = old {
                old.stop().await;
            }

            let core_path = resolve_core_path(&app, &configured_core_path(&state).await)?;
            let ip = if p.is_host {
                Some(HOST_VIRTUAL_IP)
            } else {
                None
            };
            let guest_hostname = {
                let identity = configured_network_identity(&state).await;
                if identity.is_empty() {
                    DEFAULT_GUEST_HOSTNAME.to_string()
                } else {
                    identity
                }
            };
            let hostname = match (p.is_host, p.hostname) {
                (true, Some(h)) => h,
                (true, None) => {
                    return Err(
                        "房主模式必须提供 hostname（格式 scaffolding-mc-server-{center_port}）"
                            .to_string(),
                    )
                }
                (false, h) => h.unwrap_or(guest_hostname),
            };

            let easytier = EasyTier::join(
                &core_path,
                &p.network_name,
                &p.network_secret,
                ip,
                &hostname,
                p.extra.unwrap_or_default(),
            )
            .await?;
            let rpc_portal = easytier.rpc_portal().to_string();
            let pid = easytier.pid();
            *state.easytier.lock().await = Some(easytier);
            emit_easytier_status(&app, &state).await;

            log_info!(
                "[Online] easytier 已加入网络 {}（hostname={}，rpc={}）",
                p.network_name,
                hostname,
                rpc_portal
            );
            serde_json::to_value(EasytierJoinResponse {
                success: true,
                rpc_portal,
                pid,
            })
            .map_err(|e| e.to_string())
        }),
    );

    d.register(
        "easytier_stop",
        handler!(state, app, _params, {
            let old = state.easytier.lock().await.take();
            if let Some(old) = old {
                old.stop().await;
                log_info!("[Online] easytier 已停止");
            }
            emit_easytier_status(&app, &state).await;
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "scaffolding_host_start",
        handler!(state, app, params, {
            let p: ScaffoldingHostStartParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            let (network_name, network_secret) = room_code::parse(&p.room_code)?;

            // 停止旧实例（easytier + 联机中心），保证幂等
            let old_easytier = state.easytier.lock().await.take();
            if let Some(old_easytier) = old_easytier {
                old_easytier.stop().await;
            }
            let old_server = state.scaffolding_server.lock().await.take();
            if let Some(old_server) = old_server {
                old_server.stop().await;
            }

            // MC 端口：显式参数优先，否则按游戏进程探测
            let mc_port = match p.mc_port {
                Some(port) => port,
                None => probe_mc_port(&state).await.ok_or_else(|| {
                    "未探测到 MC 局域网端口，请先在游戏内开放局域网后重试".to_string()
                })?,
            };

            // 启动联机中心（监听虚拟 IP，MC 端口写入共享状态）
            let center_state = ScaffoldingServerState::new();
            center_state.set_mc_port(Some(mc_port));
            let server = ScaffoldingServer::start_on(center_state).await?;
            let hostname = server.hostname();
            let center_port = server.port();

            // 启动 easytier（房主固定虚拟 IP + 中心 hostname）
            let core_path = resolve_core_path(&app, &configured_core_path(&state).await)?;
            let easytier = match EasyTier::join(
                &core_path,
                &network_name,
                &network_secret,
                Some(HOST_VIRTUAL_IP),
                &hostname,
                Vec::new(),
            )
            .await
            {
                Ok(e) => e,
                Err(e) => {
                    let _ = server.stop().await;
                    return Err(e);
                }
            };
            let rpc_portal = easytier.rpc_portal().to_string();
            let pid = easytier.pid();
            *state.scaffolding_server.lock().await = Some(server);
            *state.easytier.lock().await = Some(easytier);
            emit_easytier_status(&app, &state).await;

            log_info!(
                "[Online] 房主联机中心已启动: center_port={}, hostname={}, mc_port={}",
                center_port,
                hostname,
                mc_port
            );
            serde_json::to_value(ScaffoldingHostStartResponse {
                success: true,
                center_port,
                hostname,
                mc_port,
                rpc_portal,
                pid,
            })
            .map_err(|e| e.to_string())
        }),
    );

    d.register(
        "scaffolding_host_stop",
        handler!(state, app, _params, {
            let old_easytier = state.easytier.lock().await.take();
            if let Some(old_easytier) = old_easytier {
                old_easytier.stop().await;
            }
            let old_server = state.scaffolding_server.lock().await.take();
            if let Some(old_server) = old_server {
                old_server.stop().await;
            }
            emit_easytier_status(&app, &state).await;
            log_info!("[Online] 房主联机中心已停止");
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "scaffolding_client_probe",
        handler!(state, app, params, {
            let p: ScaffoldingClientProbeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            let (network_name, network_secret) = room_code::parse(&p.room_code)?;

            // 若尚未加入网络，先以房客身份（--dhcp）加入
            if state.easytier.lock().await.is_none() {
                let core_path = resolve_core_path(&app, &configured_core_path(&state).await)?;
                let hostname = {
                    let identity = configured_network_identity(&state).await;
                    if identity.is_empty() {
                        DEFAULT_GUEST_HOSTNAME.to_string()
                    } else {
                        identity
                    }
                };
                let easytier = EasyTier::join(
                    &core_path,
                    &network_name,
                    &network_secret,
                    None,
                    &hostname,
                    Vec::new(),
                )
                .await?;
                log_info!(
                    "[Online] 房客已加入网络 {}（hostname={}）",
                    network_name,
                    hostname
                );
                *state.easytier.lock().await = Some(easytier);
                emit_easytier_status(&app, &state).await;
            }

            let center_ip = p.center_ip.unwrap_or_else(|| HOST_VIRTUAL_IP.to_string());
            let center_port = p.center_port.unwrap_or(DEFAULT_CENTER_PORT);
            match scaffolding_client::discover_mc(&center_ip, center_port).await {
                Ok((mc_ip, mc_port)) => {
                    log_info!("[Online] 房客发现房主 MC 服务: {}:{}", mc_ip, mc_port);
                    serde_json::to_value(ScaffoldingClientProbeResponse {
                        success: true,
                        mc_ip,
                        mc_port,
                    })
                    .map_err(|e| e.to_string())
                }
                Err(e) => {
                    log_error!("[Online] 房客探测联机中心失败: {e}");
                    Err(e)
                }
            }
        }),
    );

    d.register(
        "easytier_status",
        handler!(state, _app, _params, {
            let guard = state.easytier.lock().await;
            serde_json::to_value(easytier_status_payload(&guard)).map_err(|e| e.to_string())
        }),
    );
}
