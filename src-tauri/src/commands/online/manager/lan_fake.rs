//! MC 局域网服务器伪装 action
//!
//! 加入方本地起 TCP 转发代理 + 周期 UDP 广播 `[MOTD]...[/MOTD][AD]port[/AD]`，
//! 本机 MC 多人游戏界面即可直接发现房主房间，进入时经代理走 TUN 桥接连房主。

use serde::{Deserialize, Serialize};
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use crate::handler;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use crate::utils::dispatcher::Dispatcher;

/// MC 局域网伪装服务（TCP 转发 + UDP 周期广播）
pub struct LanFakeServer {
    tcp_task: tokio::task::AbortHandle,
    udp_task: tokio::task::AbortHandle,
}

impl LanFakeServer {
    /// 启动伪装：TCP 监听端口 0 自动分配，UDP 绑定同端口周期广播
    pub async fn start(
        motd: String,
        target_ip: String,
        target_port: u16,
    ) -> Result<(Self, u16), String> {
        let listener = TcpListener::bind(("0.0.0.0", 0))
            .await
            .map_err(|e| format!("绑定 TCP 监听失败: {}", e))?;
        let local_port = listener
            .local_addr()
            .map_err(|e| format!("获取监听端口失败: {}", e))?
            .port();

        log_info!(
            "[Online] lan_fake 启动: port={}, target={}:{}",
            local_port,
            target_ip,
            target_port
        );

        // TCP 转发任务：接入的 MC 连接转发到房主虚拟 IP:MC 端口（走 TUN 桥接）
        let tcp_task = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut client, _)) => {
                        let ip = target_ip.clone();
                        let port = target_port;
                        tokio::spawn(async move {
                            let addr = (ip.as_str(), port);
                            match TcpStream::connect(addr).await {
                                Ok(mut target) => {
                                    let _ = copy_bidirectional(&mut client, &mut target).await;
                                }
                                Err(e) => log_warn!("[Online] lan_fake 转发目标连接失败: {}", e),
                            }
                        });
                    }
                    Err(e) => {
                        log_warn!("[Online] lan_fake TCP accept 失败: {}", e);
                        break;
                    }
                }
            }
        });

        // UDP 广播任务：每秒向 224.0.2.60:4445 广播伪装响应（MC 1.12+ 局域网发现协议）
        let msg = format!("[MOTD]{}[/MOTD][AD]{}[/AD]", motd, local_port);
        let udp_task = tokio::spawn(async move {
            let socket = match UdpSocket::bind(("0.0.0.0", local_port)).await {
                Ok(s) => s,
                Err(e) => {
                    log_error!("[Online] lan_fake UDP bind 失败: {}", e);
                    return;
                }
            };
            let _ = socket.set_multicast_loop_v4(true);
            let _ = socket.set_multicast_ttl_v4(128);
            let dest: std::net::SocketAddr = "224.0.2.60:4445".parse().expect("组播地址常量");
            loop {
                if let Err(e) = socket.send_to(msg.as_bytes(), dest).await {
                    log_warn!("[Online] lan_fake UDP 广播失败: {}", e);
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        Ok((
            Self {
                tcp_task: tcp_task.abort_handle(),
                udp_task: udp_task.abort_handle(),
            },
            local_port,
        ))
    }

    /// 停止伪装（同步 abort，不等待 task 结束）
    pub fn stop(self) {
        self.tcp_task.abort();
        self.udp_task.abort();
    }
}

/// `lan_fake_server_start` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanFakeStartParams {
    /// 多人游戏界面显示的服务器名称（MOTD）
    pub motd: String,
    /// 转发目标 IP（房主 easytier 虚拟 IP，缺省固定 `10.144.144.1`）
    #[serde(default = "default_host_virtual_ip")]
    pub target_ip: String,
    /// 转发目标端口（房主 MC 局域网端口）
    pub target_port: u16,
}

/// 房主 easytier 固定虚拟 IP
fn default_host_virtual_ip() -> String {
    crate::minecraft::online::scaffolding::easytier::HOST_VIRTUAL_IP.to_string()
}

/// `lan_fake_server_start` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanFakeStartResponse {
    pub success: bool,
    /// 实际监听的本地端口（MC 客户端将连接本机该端口）
    pub port: u16,
}

/// 注册 LAN 伪装 action 到 dispatcher
pub fn register_lan_fake_actions(d: &mut Dispatcher) {
    d.register(
        "lan_fake_server_start",
        handler!(state, _app, params, {
            let p: LanFakeStartParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;

            // 若已有伪装服务先停止（防止泄漏）
            let mut guard = state.lan_fake_server.lock().await;
            if let Some(old) = guard.take() {
                old.stop();
            }

            let (server, port) = LanFakeServer::start(p.motd, p.target_ip, p.target_port).await?;
            *guard = Some(server);

            serde_json::to_value(LanFakeStartResponse {
                success: true,
                port,
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
