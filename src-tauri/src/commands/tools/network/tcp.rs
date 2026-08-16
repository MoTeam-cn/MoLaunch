use std::time::{Duration, Instant};

use tokio::net::TcpStream;

use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::super::types::{TcpCheckParams, TcpCheckResult};

/// TCP 端口连通性检测（仅三次握手，不发送应用层数据）
///
/// 适用于 Frp 等非 Minecraft 协议服务的端口可达性检查。
/// 3 秒超时：覆盖正常 WAN RTT + TCP 握手，避免用户长时间等待。
/// 失败时 `reachable=false`，`error` 填失败原因（超时 / 拒绝 / DNS 解析失败等）。
pub async fn tcp_check(
    state: &AppState,
    params: TcpCheckParams,
) -> Result<serde_json::Value, String> {
    let _game_dir = {
        let config = state.config.lock().await;
        crate::state::resolve_game_dir(&config.game_dir)
    };

    log_info!("[TcpCheck] check {}:{}", params.host, params.port);

    let result = check_tcp(&params.host, params.port).await;
    if result.reachable {
        log_info!(
            "[TcpCheck] success: {}:{} latency={}ms",
            params.host,
            params.port,
            result.latency_ms
        );
    } else {
        log_warn!(
            "[TcpCheck] failed: {}:{} err={}",
            params.host,
            params.port,
            result.error
        );
    }

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 执行 TCP 连接检测（3 秒超时），供 tcp_check 与地址延迟测试（tcping）复用
pub(crate) async fn check_tcp(host: &str, port: u16) -> TcpCheckResult {
    let start = Instant::now();
    match tokio::time::timeout(Duration::from_secs(3), TcpStream::connect((host, port))).await {
        Ok(Ok(_stream)) => TcpCheckResult {
            reachable: true,
            latency_ms: start.elapsed().as_millis() as u64,
            error: String::new(),
        },
        Ok(Err(e)) => TcpCheckResult {
            reachable: false,
            latency_ms: 0,
            error: format!("连接失败: {}", e),
        },
        Err(_) => TcpCheckResult {
            reachable: false,
            latency_ms: 0,
            error: format!("连接超时 (3s): {}:{}", host, port),
        },
    }
}
