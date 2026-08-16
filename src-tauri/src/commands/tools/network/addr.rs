//! 地址延迟测试（tcp 握手 / udp 探针 / 系统 ping，支持持续模式经事件推送）
//!
//! 一次性（persistent=false）：并发测试所有目标并返回首轮结果；
//! 持续（persistent=true）：先返回首轮结果，同时 spawn 后台任务按 interval_ms 周期
//! 测试并经 `tools-latency-update` 事件推送，直到 `address_latency_stop` 停止。

use std::time::{Duration, Instant};

use futures_util::future::join_all;
use tauri::Emitter;

use crate::log_info;
use crate::state::AppState;

use super::super::types::{
    AddressLatencyItem, AddressLatencyResult, AddressLatencyTestParams, AddressTarget,
};
use super::tcp::check_tcp;

/// 持续延迟测试 emit 事件名（payload = AddressLatencyResult）
pub const LATENCY_UPDATE_EVENT: &str = "tools-latency-update";
/// 单轮测试超时（tcp 连接 / ping 命令执行）
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
/// UDP 探针等待对端回包超时
const UDP_RECV_TIMEOUT: Duration = Duration::from_secs(1);

/// 地址延迟测试入口
pub async fn address_latency_test(
    state: &AppState,
    app: tauri::AppHandle,
    params: AddressLatencyTestParams,
) -> Result<serde_json::Value, String> {
    let task_id = if params.persistent {
        // 替换并停止上一轮持续任务（同一时刻仅一个）
        if let Some(old) = state.latency_test_task.lock().await.take() {
            old.abort();
        }
        let targets = params.targets.clone();
        let interval = Duration::from_millis(params.interval_ms.max(1000));
        let app_handle = app.clone();
        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let result = run_tests(&targets).await;
                let _ = app_handle.emit(LATENCY_UPDATE_EVENT, &result);
            }
        })
        .abort_handle();
        *state.latency_test_task.lock().await = Some(handle);
        Some(format!(
            "latency-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ))
    } else {
        None
    };

    let result = run_tests(&params.targets).await;
    log_info!(
        "[AddressLatency] 测试 {} 个目标, persistent={}",
        result.results.len(),
        params.persistent
    );
    serde_json::to_value(AddressLatencyResult {
        results: result.results,
        task_id,
    })
    .map_err(|e| e.to_string())
}

/// 停止持续延迟测试任务
pub async fn address_latency_stop(state: &AppState) -> Result<serde_json::Value, String> {
    if let Some(handle) = state.latency_test_task.lock().await.take() {
        handle.abort();
        log_info!("[AddressLatency] 持续测试已停止");
    }
    Ok(serde_json::json!({}))
}

/// 并发测试全部目标
async fn run_tests(targets: &[AddressTarget]) -> AddressLatencyResult {
    let futures = targets.iter().map(check_target);
    let results = join_all(futures).await;
    AddressLatencyResult {
        results,
        task_id: None,
    }
}

/// 按协议执行单目标延迟测试
async fn check_target(target: &AddressTarget) -> AddressLatencyItem {
    match target.protocol.as_str() {
        "udp" => check_udp(target).await,
        "ping" => check_ping(target).await,
        _ => check_tcp_item(target).await,
    }
}

/// TCP 握手延迟（tcping），复用 tcp.rs 的 check_tcp
async fn check_tcp_item(target: &AddressTarget) -> AddressLatencyItem {
    let result = check_tcp(&target.host, target.port).await;
    if result.reachable {
        item(target, true, result.latency_ms, "")
    } else {
        item(target, false, 0, &result.error)
    }
}

/// UDP 探针延迟：connect + 发 1 字节并等待对端回包
async fn check_udp(target: &AddressTarget) -> AddressLatencyItem {
    let addr = match tokio::net::lookup_host((target.host.as_str(), target.port)).await {
        Ok(mut it) => match it.next() {
            Some(a) => a,
            None => return item(target, false, 0, "DNS 解析无结果"),
        },
        Err(e) => return item(target, false, 0, &format!("DNS 解析失败: {e}")),
    };
    let sock = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => return item(target, false, 0, &format!("创建 UDP socket 失败: {e}")),
    };
    if let Err(e) = sock.connect(addr).await {
        return item(target, false, 0, &format!("UDP connect 失败: {e}"));
    }
    let start = Instant::now();
    let mut buf = [0u8; 16];
    let _ = sock.send(&[0u8]).await;
    match tokio::time::timeout(UDP_RECV_TIMEOUT, sock.recv(&mut buf)).await {
        Ok(Ok(_)) => item(target, true, start.elapsed().as_millis() as u64, ""),
        Ok(Err(e)) => item(target, false, 0, &format!("UDP 接收失败: {e}")),
        Err(_) => item(target, false, 0, "UDP 探针超时（对端无回包）"),
    }
}

/// ICMP ping 延迟（调用系统 ping 命令并解析 RTT）
async fn check_ping(target: &AddressTarget) -> AddressLatencyItem {
    let mut cmd = tokio::process::Command::new("ping");
    #[cfg(windows)]
    {
        cmd.arg("-n")
            .arg("1")
            .arg("-w")
            .arg("3000")
            .arg(&target.host);
    }
    #[cfg(not(windows))]
    {
        cmd.arg("-c").arg("1").arg("-W").arg("3").arg(&target.host);
    }
    let output = match tokio::time::timeout(PROBE_TIMEOUT, cmd.output()).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => return item(target, false, 0, &format!("执行 ping 失败: {e}")),
        Err(_) => return item(target, false, 0, "ping 执行超时"),
    };
    if !output.status.success() {
        return item(target, false, 0, "ping 失败（主机不可达或禁 ICMP）");
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    match parse_ping_rtt(&text) {
        Some(ms) => item(target, true, ms, ""),
        // 命令成功但输出格式未识别（如 time<1ms 或本地回环），按可达处理
        None => item(target, true, 0, ""),
    }
}

/// 从 ping 输出解析 RTT 毫秒（兼容中英文：`time=12.3 ms` / `时间=12ms` / `time<1ms`）
fn parse_ping_rtt(output: &str) -> Option<u64> {
    for line in output.lines() {
        let (pos, _) = if let Some(p) = line.find("time=") {
            (p + 5, 5)
        } else if let Some(p) = line.find("时间=") {
            (p + 3, 3)
        } else {
            continue;
        };
        let rest = &line[pos..];
        if rest.starts_with('<') {
            // time<1ms：不足 1ms
            return Some(1);
        }
        let digits: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Some(ms) = digits.split('.').next().and_then(|s| s.parse::<u64>().ok()) {
            return Some(ms);
        }
    }
    None
}

fn item(
    target: &AddressTarget,
    reachable: bool,
    latency_ms: u64,
    error: &str,
) -> AddressLatencyItem {
    AddressLatencyItem {
        name: target.name.clone(),
        host: target.host.clone(),
        port: target.port,
        protocol: target.protocol.clone(),
        reachable,
        latency_ms,
        error: error.to_string(),
    }
}

#[cfg(test)]
#[path = "addr_test.rs"]
mod tests;
