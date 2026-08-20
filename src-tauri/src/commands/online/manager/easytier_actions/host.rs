//! 房主联机动作（一站式启动 / 停止 / 手动端口 / 白名单重建）

use std::collections::HashSet;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::commands::online::manager::signaling_manager::host_heartbeat_now;
use crate::handler;
use crate::log_debug;
use crate::log_info;
use crate::minecraft::online::scaffolding::code as room_code;
use crate::minecraft::online::scaffolding::easytier::{EasyTier, HOST_VIRTUAL_IP};
use crate::minecraft::online::scaffolding::server::{
    ScaffoldingServer, ScaffoldingServerState, CENTER_HOSTNAME_PREFIX,
};
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;

use super::watch::host_watch_loop;
use super::{
    configured_core_path, configured_easytier_peers, emit_easytier_status, resolve_cli_path,
    resolve_core_path,
};

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

/// `scaffolding_host_start` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingHostStartResponse {
    pub success: bool,
    /// 联机中心实际监听端口
    pub center_port: u16,
    /// 中心 hostname（`scaffolding-mc-server-{center_port}`）
    pub hostname: String,
    /// 房主 MC 局域网端口（先开房后开局域网时为 None，后台监视发现端口后自动更新）
    pub mc_port: Option<u16>,
    /// rpc-portal 地址
    pub rpc_portal: String,
    /// easytier 子进程 PID
    pub pid: Option<u32>,
}

/// `scaffolding_host_set_mc_port` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingHostSetMcPortParams {
    /// 手动指定的 MC 端口（None 清除手动覆盖，恢复自动探测）
    #[serde(default)]
    pub mc_port: Option<u16>,
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

/// 房主 easytier no-tun 白名单参数（允许虚拟网络流量注入本机端口）。
///
/// 联机中心（TCP）+ MC 端口（TCP/UDP）逗号分隔；MC 端口未知时仅放行联机中心，
/// 待监视循环发现端口后经 `rebuild_host_easytier` 重建。
fn host_whitelist_args(center_port: u16, mc_port: Option<u16>) -> Vec<String> {
    match mc_port {
        Some(mc_port) => vec![
            "--tcp-whitelist".to_string(),
            format!("{center_port},{mc_port}"),
            "--udp-whitelist".to_string(),
            mc_port.to_string(),
        ],
        None => vec!["--tcp-whitelist".to_string(), center_port.to_string()],
    }
}

/// 以最新 MC 端口重建房主 easytier（no-tun + 白名单），供监视循环端口变化时调用。
///
/// 先停旧进程再以新白名单 join；失败时 easytier 置空（监视循环随之退出）。
pub(super) async fn rebuild_host_easytier(
    state: &AppState,
    app: &tauri::AppHandle,
    mc_port: Option<u16>,
) -> Result<(), String> {
    let center_port = state
        .scaffolding_server
        .lock()
        .await
        .as_ref()
        .map(|s| s.port())
        .ok_or_else(|| "联机中心未运行".to_string())?;
    let (network_name, network_secret) = state
        .host_network_cred
        .lock()
        .await
        .clone()
        .ok_or_else(|| "房主网络凭据缺失".to_string())?;
    let core_path = resolve_core_path(state, app, &configured_core_path(state).await).await?;
    let cli_path = resolve_cli_path(&core_path);
    let mut extra = configured_easytier_peers(state).await;
    extra.extend(host_whitelist_args(center_port, mc_port));
    let easytier = EasyTier::join(
        &core_path,
        &cli_path,
        &network_name,
        &network_secret,
        Some(HOST_VIRTUAL_IP),
        &format!("{CENTER_HOSTNAME_PREFIX}{center_port}"),
        extra,
        true,
    )
    .await?;
    let old = state.easytier.lock().await.take();
    if let Some(old) = old {
        old.stop().await;
    }
    *state.easytier.lock().await = Some(easytier);
    emit_easytier_status(app, state).await;
    log_debug!("[Online] 房主 easytier 已重建（no-tun 白名单，mc_port={mc_port:?}）");
    Ok(())
}

/// 房主成员监听循环：每 5s 比对 easytier 在线节点（过滤中继，排除本机），
/// 出现新成员时立即心跳上报一次（不等 2 分钟定时，也不打断其队列），并推送
/// `easytier-status` 事件供前端实时刷新组网列表。easytier 被停止（置 None）时退出。
///
/// 首次快照视为全新增：开房后立即上报一次当前人数，后续仅新增节点时触发。
async fn host_member_heartbeat_loop(state: AppState, app: tauri::AppHandle, room_code: String) {
    let mut last: HashSet<String> = HashSet::new();
    loop {
        let current = {
            let guard = state.easytier.lock().await;
            match &*guard {
                Some(et) => et.peers().await.ok().map(|list| {
                    list.into_iter()
                        .filter(|p| !p.is_self)
                        .map(|p| p.hostname.clone())
                        .collect::<HashSet<_>>()
                }),
                None => None,
            }
        };
        let Some(current) = current else {
            return;
        };
        let joined = !current.is_empty() && current.difference(&last).next().is_some();
        last = current;
        if joined {
            log_info!("[Online] 检测到新成员加入虚拟网络，立即心跳上报");
            if let Err(e) = host_heartbeat_now(&state, &room_code).await {
                log_debug!("[Online] 成员加入心跳失败（不影响后续监听）: {e}");
            }
            emit_easytier_status(&app, &state).await;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

/// 注册房主动作到 dispatcher
pub(super) fn register_host(d: &mut Dispatcher) {
    d.register(
        "scaffolding_host_start",
        handler!(state, app, params, {
            let p: ScaffoldingHostStartParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            let (network_name, network_secret) = room_code::parse(&p.room_code)?;

            // 停止旧实例（后台监视 + 成员监听 + easytier + 联机中心），保证幂等
            let old_watch = state.scaffolding_host_watch.lock().await.take();
            if let Some(old_watch) = old_watch {
                old_watch.abort();
            }
            let old_heartbeat = state.scaffolding_heartbeat.lock().await.take();
            if let Some(old_heartbeat) = old_heartbeat {
                old_heartbeat.abort();
            }
            *state.manual_mc_port.lock().await = None;
            let old_easytier = state.easytier.lock().await.take();
            if let Some(old_easytier) = old_easytier {
                old_easytier.stop().await;
            }
            let old_server = state.scaffolding_server.lock().await.take();
            if let Some(old_server) = old_server {
                old_server.stop().await;
            }

            // MC 端口：显式参数优先，其次按游戏进程探测；均未获知时允许先开房，
            // 由后台监视循环发现端口后自动回写（host_watch_loop）
            let mc_port = match p.mc_port {
                Some(port) => Some(port),
                None => probe_mc_port(&state).await,
            };

            // 启动联机中心（监听虚拟 IP，MC 端口写入共享状态）
            let center_state = ScaffoldingServerState::new();
            center_state.set_mc_port(mc_port);
            let server = ScaffoldingServer::start_on(center_state.clone()).await?;
            let hostname = server.hostname();
            let center_port = server.port();

            // 启动 easytier（房主固定虚拟 IP + 中心 hostname + no-tun 白名单）
            let core_path =
                resolve_core_path(&state, &app, &configured_core_path(&state).await).await?;
            let cli_path = resolve_cli_path(&core_path);
            let mut extra = Vec::new();
            extra.extend(configured_easytier_peers(&state).await);
            extra.extend(host_whitelist_args(center_port, mc_port));
            let easytier = match EasyTier::join(
                &core_path,
                &cli_path,
                &network_name,
                &network_secret,
                Some(HOST_VIRTUAL_IP),
                &hostname,
                extra,
                true,
            )
            .await
            {
                Ok(e) => e,
                Err(e) => {
                    let _ = server.stop().await;
                    return Err(e);
                }
            };
            // 记录网络凭据供监视循环按 MC 端口变化重建白名单
            *state.host_network_cred.lock().await = Some((network_name, network_secret));
            let rpc_portal = easytier.rpc_portal().to_string();
            let pid = easytier.pid();
            *state.scaffolding_server.lock().await = Some(server);
            *state.easytier.lock().await = Some(easytier);
            emit_easytier_status(&app, &state).await;

            // 启动后台监视循环（端口热更新 + 自动关房）
            let watch_center = center_state.clone();
            let watch_app = app.clone();
            let watch_state = state.clone();
            let watch = tokio::spawn(async move {
                host_watch_loop(watch_center, watch_app, watch_state).await;
            });
            *state.scaffolding_host_watch.lock().await = Some(watch.abort_handle());

            // 启动成员监听任务：新成员加入虚拟网络即心跳上报（不打断 2 分钟定时）
            let hb_state = state.clone();
            let hb_app = app.clone();
            let hb_room = p.room_code.clone();
            let hb = tokio::spawn(async move {
                host_member_heartbeat_loop(hb_state, hb_app, hb_room).await;
            });
            *state.scaffolding_heartbeat.lock().await = Some(hb.abort_handle());

            log_info!(
                "[Online] 房主联机中心已启动: center_port={}, hostname={}, mc_port={:?}",
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
            // 中止后台监视与成员监听任务，清除手动端口覆盖
            let watch = state.scaffolding_host_watch.lock().await.take();
            if let Some(watch) = watch {
                watch.abort();
            }
            let heartbeat = state.scaffolding_heartbeat.lock().await.take();
            if let Some(heartbeat) = heartbeat {
                heartbeat.abort();
            }
            *state.manual_mc_port.lock().await = None;
            *state.host_network_cred.lock().await = None;
            *state.client_port_forwards.lock().await = Vec::new();
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
        "scaffolding_host_set_mc_port",
        handler!(state, _app, params, {
            let p: ScaffoldingHostSetMcPortParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            *state.manual_mc_port.lock().await = p.mc_port;
            // 手动端口立即生效（联机中心直接回写）；清除时由监视循环自动补回
            if let Some(server) = state.scaffolding_server.lock().await.as_ref() {
                server.state().set_mc_port(p.mc_port);
            }
            log_debug!("[Online] 房主手动 MC 端口: {:?}", p.mc_port);
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );
}
