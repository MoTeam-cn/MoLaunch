//! MC 局域网伪装 action
//!
//! no-tun 下仅承担发现广播：周期向 `224.0.2.60:4445` 发送 `[MOTD]...[/MOTD][AD]port[/AD]`，
//! 本机 MC 多人游戏界面直接发现房主房间；进服流量由 easytier port-forward
//! 在 `127.0.0.1:{port}` 承担，本模块不再做 TCP 转发。

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

use crate::handler;
use crate::log_info;
use crate::log_warn;
use crate::utils::dispatcher::Dispatcher;

/// MC 局域网伪装服务（纯 UDP 周期广播）
pub struct LanFakeServer {
    udp_task: tokio::task::AbortHandle,
}

impl LanFakeServer {
    /// 启动伪装：UDP 绑定 `port` 周期广播 `[AD]port[/AD]`（port 为本地 port-forward 端口）
    pub async fn start(motd: String, port: u16) -> Result<Self, String> {
        let socket = UdpSocket::bind(("0.0.0.0", port))
            .await
            .map_err(|e| format!("lan_fake UDP bind 失败: {e}"))?;
        let _ = socket.set_multicast_loop_v4(true);
        let _ = socket.set_multicast_ttl_v4(128);
        let dest: std::net::SocketAddr = "224.0.2.60:4445".parse().expect("组播地址常量");

        log_info!("[Online] lan_fake 广播启动: port={port}");
        let msg = format!("[MOTD]{motd}[/MOTD][AD]{port}[/AD]");
        let udp_task = tokio::spawn(async move {
            loop {
                if let Err(e) = socket.send_to(msg.as_bytes(), dest).await {
                    log_warn!("[Online] lan_fake UDP 广播失败: {e}");
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        Ok(Self {
            udp_task: udp_task.abort_handle(),
        })
    }

    /// 停止伪装（同步 abort，不等待 task 结束）
    pub fn stop(self) {
        self.udp_task.abort();
    }
}

/// `lan_fake_server_start` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanFakeStartParams {
    /// 多人游戏界面显示的服务器名称（MOTD）
    pub motd: String,
    /// 进服端口（本地 port-forward 端口，MC 客户端连接 `127.0.0.1:{port}`）
    pub port: u16,
}

/// `lan_fake_server_start` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanFakeStartResponse {
    pub success: bool,
    /// 广播进服端口
    pub port: u16,
}

/// 注册 LAN 伪装 action 到 dispatcher
pub fn register_lan_fake_actions(d: &mut Dispatcher) {
    d.register(
        "lan_fake_server_start",
        handler!(state, _app, params, {
            let p: LanFakeStartParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;

            // 若已有伪装服务先停止（防止泄漏）
            let mut guard = state.lan_fake_server.lock().await;
            if let Some(old) = guard.take() {
                old.stop();
            }

            let server = LanFakeServer::start(p.motd, p.port).await?;
            *guard = Some(server);

            serde_json::to_value(LanFakeStartResponse {
                success: true,
                port: p.port,
            })
            .map_err(|e| e.to_string())
        }),
    );

    d.register(
        "lan_fake_server_stop",
        handler!(state, _app, _params, {
            let mut guard = state.lan_fake_server.lock().await;
            if let Some(server) = guard.take() {
                server.stop();
                log_info!("[Online] lan_fake 已停止");
            }
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );
}
