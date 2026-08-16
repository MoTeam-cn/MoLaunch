//! 地址延迟测试（tcp 握手 / udp 探针 / icmp ping）
//!
//! 一次性并发测试所有目标并返回结果；ICMP ping 自实现（见 icmp 模块），
//! 不依赖系统 ping 命令，无编码与输出格式差异问题。

use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};

use futures_util::future::join_all;

use crate::log_info;

use super::super::types::{
    AddressLatencyItem, AddressLatencyResult, AddressLatencyTestParams, AddressTarget,
};
use super::icmp::ping_once;
use super::tcp::check_tcp;

/// 单轮测试超时（tcp 连接 / icmp ping / udp 探针）
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
/// UDP 探针等待对端回包超时
const UDP_RECV_TIMEOUT: Duration = Duration::from_secs(1);

/// 地址延迟测试入口
pub async fn address_latency_test(
    params: AddressLatencyTestParams,
) -> Result<serde_json::Value, String> {
    let results = run_tests(&params.targets).await;
    log_info!("[AddressLatency] 测试 {} 个目标", results.len());
    serde_json::to_value(AddressLatencyResult { results }).map_err(|e| e.to_string())
}

/// 并发测试全部目标
async fn run_tests(targets: &[AddressTarget]) -> Vec<AddressLatencyItem> {
    let futures = targets.iter().map(check_target);
    join_all(futures).await
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
    let addrs = match tokio::net::lookup_host((target.host.as_str(), target.port)).await {
        Ok(it) => it.collect::<Vec<_>>(),
        Err(e) => return item(target, false, 0, &format!("DNS 解析失败: {e}")),
    };
    // 优先 IPv4（与 ping / tcp 一致），无 IPv4 时才用 IPv6
    let addr = match addrs.iter().find(|a| a.is_ipv4()).or_else(|| addrs.first()) {
        Some(a) => *a,
        None => return item(target, false, 0, "DNS 解析无结果"),
    };
    // socket 地址族必须与目标一致，否则 Windows 报 os error 10047（协议不兼容）
    let bind_addr = if addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let sock = match tokio::net::UdpSocket::bind(bind_addr).await {
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

/// ICMP ping 延迟（自实现：DNS 解析取 IPv4 后发送 ICMP Echo，不依赖系统 ping）
async fn check_ping(target: &AddressTarget) -> AddressLatencyItem {
    let ip = match resolve_ipv4(&target.host).await {
        Ok(ip) => ip,
        Err(e) => return item(target, false, 0, &e),
    };
    match ping_once(ip, PROBE_TIMEOUT).await {
        Ok(rtt) => item(target, true, rtt.as_millis() as u64, ""),
        Err(e) => item(target, false, 0, &e),
    }
}

/// 解析主机名为 IPv4（IP 字面量直接返回；当前仅支持 IPv4 探测）
async fn resolve_ipv4(host: &str) -> Result<Ipv4Addr, String> {
    let addrs = tokio::net::lookup_host((host, 0))
        .await
        .map_err(|e| format!("DNS 解析失败: {e}"))?;
    for addr in addrs {
        if let IpAddr::V4(ip) = addr.ip() {
            return Ok(ip);
        }
    }
    Err("DNS 解析无 IPv4 地址（当前仅支持 IPv4 探测）".to_string())
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
