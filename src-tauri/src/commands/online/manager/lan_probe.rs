//! MC 局域网端口探测 action（与 MC 多人游戏发现房间同源）
//!
//! 监听 UDP 多播 224.0.2.60:4445，解析 MC 服务器周期广播的
//! `[MOTD]...[/MOTD][AD]port[/AD]`，得到局域网服务实际端口。
//! 房主可探测自己 MC 服务器端口；加入方可探测本地伪装代理端口。

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

use crate::handler;
use crate::log_info;
use crate::log_warn;
use crate::utils::dispatcher::Dispatcher;

/// MC 局域网发现多播地址（1.12+）
const LAN_DISCOVERY_ADDR: (&str, u16) = ("224.0.2.60", 4445);

/// `lan_port_probe` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanPortProbeParams {
    /// 监听时长（毫秒），默认 6000，上限 15000
    pub timeout_ms: Option<u64>,
}

/// `lan_port_probe` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanPortProbeResponse {
    pub success: bool,
    /// 解析出的 MC 局域网端口（0 = 未检测到）
    pub port: u16,
    /// 广播中的 MOTD 文本
    pub motd: String,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 解析广播 `[MOTD]xx[/MOTD][AD]port[/AD]` → (motd, port)
fn parse_lan_broadcast(raw: &str) -> Option<(String, u16)> {
    let motd_start = raw.find("[MOTD]")? + "[MOTD]".len();
    let motd_end = raw.find("[/MOTD]")?;
    let ad_start = raw.find("[AD]")? + "[AD]".len();
    let ad_end = raw.find("[/AD]")?;
    if motd_start >= motd_end || ad_start >= ad_end {
        return None;
    }
    let port = raw[ad_start..ad_end].parse::<u16>().ok()?;
    Some((raw[motd_start..motd_end].to_string(), port))
}

/// 注册局域网端口探测 action 到 dispatcher
pub fn register_lan_probe_actions(d: &mut Dispatcher) {
    d.register(
        "lan_port_probe",
        handler!(_state, _app, params, {
            let p: LanPortProbeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let timeout_ms = p.timeout_ms.unwrap_or(6000).min(15000);

            // 绑定发现端口并加入多播组（与 MC 多人游戏发现的监听方式一致）
            let socket = std::net::UdpSocket::bind(("0.0.0.0", LAN_DISCOVERY_ADDR.1))
                .map_err(|e| format!("绑定局域网发现端口 {} 失败: {}", LAN_DISCOVERY_ADDR.1, e))?;
            let _ = socket.set_multicast_loop_v4(true);
            let group: std::net::Ipv4Addr = LAN_DISCOVERY_ADDR.0.parse().expect("多播地址常量");
            let any: std::net::Ipv4Addr = "0.0.0.0".parse().expect("通配地址常量");
            socket
                .join_multicast_v4(&group, &any)
                .map_err(|e| format!("加入多播组失败: {}", e))?;
            let socket = UdpSocket::from_std(socket).map_err(|e| e.to_string())?;

            log_info!("[Online] lan_port_probe: 监听局域网广播 {}ms", timeout_ms);

            let deadline =
                tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
            let mut buf = [0u8; 1024];
            let mut last_err = String::new();
            let mut resp = LanPortProbeResponse {
                success: false,
                port: 0,
                motd: String::new(),
                error: String::new(),
            };
            loop {
                let remain = deadline.saturating_duration_since(tokio::time::Instant::now());
                if remain.is_zero() {
                    break;
                }
                match tokio::time::timeout(remain, socket.recv_from(&mut buf)).await {
                    Ok(Ok((len, _src))) => {
                        let Ok(text) = std::str::from_utf8(&buf[..len]) else {
                            continue;
                        };
                        if let Some((motd, port)) = parse_lan_broadcast(text) {
                            log_info!(
                                "[Online] lan_port_probe 检测到广播: port={}, motd={}",
                                port,
                                motd
                            );
                            resp.success = true;
                            resp.port = port;
                            resp.motd = motd;
                            break;
                        }
                    }
                    Ok(Err(e)) => {
                        last_err = format!("接收失败: {}", e);
                        break;
                    }
                    Err(_) => break,
                }
            }

            if !resp.success {
                resp.error = if last_err.is_empty() {
                    "未检测到局域网广播，请确认 Minecraft 已开放局域网".to_string()
                } else {
                    log_warn!("[Online] lan_port_probe: {}", last_err);
                    last_err
                };
            }
            serde_json::to_value(resp).map_err(|e| e.to_string())
        }),
    );
}

#[cfg(test)]
#[path = "lan_probe_test.rs"]
mod lan_probe_test;
