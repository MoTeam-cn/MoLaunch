//! frpc 状态查询：所有隧道状态 + 单隧道状态

use crate::commands::frp::tunnel;
use crate::commands::frp::{TunnelStatus, TunnelWithStatus};

use super::RUNNING;

/// 查询所有隧道状态（附加运行状态 + PID）
pub async fn list_tunnels_with_status() -> Result<Vec<TunnelWithStatus>, String> {
    let tunnels = tunnel::list_tunnels().await?;
    let running = RUNNING.lock().await;

    let result = tunnels
        .into_iter()
        .map(|t| {
            let (status, pid) = if running.contains_key(&t.id) {
                let pid = running.get(&t.id).map(|h| h.pid);
                (TunnelStatus::Running, pid)
            } else {
                (TunnelStatus::Stopped, None)
            };
            TunnelWithStatus {
                tunnel: t,
                status,
                pid,
            }
        })
        .collect();

    Ok(result)
}

/// 查询单个隧道状态
pub async fn get_tunnel_status(id: String) -> Result<TunnelStatus, String> {
    let running = RUNNING.lock().await;
    if running.contains_key(&id) {
        Ok(TunnelStatus::Running)
    } else {
        Ok(TunnelStatus::Stopped)
    }
}
