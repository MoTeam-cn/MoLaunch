//! 房客联机动作（加入网络 / 探测房主 / 轮询 / port-forward）

use serde::{Deserialize, Serialize};

use crate::handler;
use crate::log_debug;
use crate::log_info;
use crate::minecraft::online::scaffolding::client as scaffolding_client;
use crate::minecraft::online::scaffolding::code as room_code;
use crate::minecraft::online::scaffolding::easytier::{pick_free_port, EasyTier, PortForwardRule};
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;
use tokio::net::TcpListener;

use super::{
    configured_core_path, configured_easytier_peers, configured_network_identity,
    emit_easytier_status, resolve_cli_path, resolve_core_path, DEFAULT_GUEST_HOSTNAME,
};

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

/// `scaffolding_client_probe` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingClientProbeResponse {
    pub success: bool,
    /// 进服连接地址 IP（no-tun 下固定为本机 `127.0.0.1`，指向本地 port-forward）
    pub mc_ip: String,
    /// 进服连接端口（本地 port-forward 端口，尽力与房主 MC 端口相同）
    pub mc_port: u16,
}

/// 申请本地转发端口：优先复用 mc_port（MC 感知的服务端口不变），被占则随机空闲端口
async fn pick_local_port(mc_port: u16) -> Result<u16, String> {
    if TcpListener::bind(("0.0.0.0", mc_port)).await.is_ok() {
        return Ok(mc_port);
    }
    pick_free_port().await
}

/// 确保房客 port-forward 规则指向 `mc_ip:mc_port`；目标变化时移除旧规则并重建。
///
/// 返回本地转发端口（尽力与 mc_port 相同，供进服地址 `127.0.0.1:{local_port}` 使用）。
async fn ensure_guest_port_forwards(
    state: &AppState,
    mc_ip: &str,
    mc_port: u16,
) -> Result<u16, String> {
    let desired_dst = format!("{mc_ip}:{mc_port}");
    {
        let rules = state.client_port_forwards.lock().await;
        if let Some(rule) = rules.iter().find(|r| r.dst_addr == desired_dst) {
            let local_port = rule
                .bind_addr
                .rsplit(':')
                .next()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(mc_port);
            return Ok(local_port);
        }
    }
    // 目标变化：先移除旧规则（进程存活期间经 RPC 清理）
    let old_rules = {
        let mut rules = state.client_port_forwards.lock().await;
        std::mem::take(&mut *rules)
    };
    {
        let guard = state.easytier.lock().await;
        if let Some(easytier) = guard.as_ref() {
            for rule in &old_rules {
                let _ = easytier
                    .remove_port_forward(&rule.proto, &rule.bind_addr)
                    .await;
            }
        }
    }
    // 建立 TCP + UDP 两条规则（本地监听 → 房主虚拟 IP）
    let local_port = pick_local_port(mc_port).await?;
    let bind_addr = format!("0.0.0.0:{local_port}");
    {
        let guard = state.easytier.lock().await;
        let easytier = guard
            .as_ref()
            .ok_or_else(|| "easytier 未加入网络".to_string())?;
        easytier
            .add_port_forward("tcp", &bind_addr, &desired_dst)
            .await?;
        easytier
            .add_port_forward("udp", &bind_addr, &desired_dst)
            .await?;
    }
    let mut rules = state.client_port_forwards.lock().await;
    rules.push(PortForwardRule {
        proto: "tcp".into(),
        bind_addr: bind_addr.clone(),
        dst_addr: desired_dst.clone(),
    });
    rules.push(PortForwardRule {
        proto: "udp".into(),
        bind_addr,
        dst_addr: desired_dst.clone(),
    });
    log_debug!("[Online] 房客 port-forward 已建立: 127.0.0.1:{local_port} -> {desired_dst}");
    Ok(local_port)
}

/// 注册房客动作到 dispatcher
pub(super) fn register_guest(d: &mut Dispatcher) {
    d.register(
        "scaffolding_client_probe",
        handler!(state, app, params, {
            let p: ScaffoldingClientProbeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            let (network_name, network_secret) = room_code::parse(&p.room_code)?;

            // 若尚未加入网络，先以房客身份（--dhcp，no-tun）加入
            if state.easytier.lock().await.is_none() {
                let core_path =
                    resolve_core_path(&state, &app, &configured_core_path(&state).await).await?;
                let cli_path = resolve_cli_path(&core_path);
                let hostname = {
                    let identity = configured_network_identity(&state).await;
                    if identity.is_empty() {
                        DEFAULT_GUEST_HOSTNAME.to_string()
                    } else {
                        identity
                    }
                };
                let mut extra = Vec::new();
                extra.extend(configured_easytier_peers(&state).await);
                let easytier = EasyTier::join(
                    &core_path,
                    &cli_path,
                    &network_name,
                    &network_secret,
                    None,
                    &hostname,
                    extra,
                    true,
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

            // 解析联机中心地址：显式参数优先，否则经 easytier-cli 从虚拟网络自动发现
            let (center_ip, center_port) = {
                let guard = state.easytier.lock().await;
                let easytier = guard
                    .as_ref()
                    .ok_or_else(|| "easytier 未加入网络".to_string())?;
                scaffolding_client::resolve_center_addr(
                    p.center_ip.as_deref(),
                    p.center_port,
                    easytier,
                )
                .await?
            };
            // no-tun 下系统栈无法直连虚拟 IP：先建联机中心本地转发，再经本地端口探测
            let center_local = ensure_guest_port_forwards(&state, &center_ip, center_port).await?;
            let mc_port = scaffolding_client::discover_mc_at("127.0.0.1", center_local).await?;
            // 进服转发：MC 端口与联机中心相同则复用本地端口，否则单独建立
            let local_port = if mc_port == center_port {
                center_local
            } else {
                ensure_guest_port_forwards(&state, &center_ip, mc_port).await?
            };
            log_info!(
                "[Online] 房客发现房主 MC 服务: {}:{}（本地转发 127.0.0.1:{}）",
                center_ip,
                mc_port,
                local_port
            );
            serde_json::to_value(ScaffoldingClientProbeResponse {
                success: true,
                mc_ip: "127.0.0.1".to_string(),
                mc_port: local_port,
            })
            .map_err(|e| e.to_string())
        }),
    );

    d.register(
        "scaffolding_client_poll",
        handler!(state, _app, params, {
            let _p: ScaffoldingClientProbeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            // 轻量轮询：不 join，仅解析中心地址后探测 MC 端口（复用 probe 发现逻辑）
            let (center_ip, center_port) = {
                let guard = state.easytier.lock().await;
                let easytier = guard
                    .as_ref()
                    .ok_or_else(|| "easytier 未加入网络".to_string())?;
                scaffolding_client::resolve_center_addr(None, None, easytier).await?
            };
            // no-tun 下先建联机中心本地转发，再经本地端口探测
            let center_local = ensure_guest_port_forwards(&state, &center_ip, center_port).await?;
            let mc_port = scaffolding_client::discover_mc_at("127.0.0.1", center_local).await?;
            // 端口变化时经 ensure_guest_port_forwards 重建本地转发规则
            let local_port = if mc_port == center_port {
                center_local
            } else {
                ensure_guest_port_forwards(&state, &center_ip, mc_port).await?
            };
            serde_json::to_value(ScaffoldingClientProbeResponse {
                success: true,
                mc_ip: "127.0.0.1".to_string(),
                mc_port: local_port,
            })
            .map_err(|e| e.to_string())
        }),
    );
}
