//! Scaffolding 房客发现流程：解析联机中心地址，连接后依次执行 c:ping / c:protocols / c:server_port。

use crate::minecraft::online::scaffolding::easytier::EasyTier;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// 单次请求/响应超时
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// 探测指纹（c:ping 回显校验）
const FINGERPRINT: &[u8] = b"mo-launch";

/// 发送请求并读取响应
async fn send_request(
    stream: &mut TcpStream,
    kind: &str,
    body: &[u8],
) -> Result<(u8, Vec<u8>), String> {
    stream
        .write_all(&[kind.len() as u8])
        .await
        .map_err(|e| format!("发送请求类型失败: {e}"))?;
    stream
        .write_all(kind.as_bytes())
        .await
        .map_err(|e| format!("发送请求类型失败: {e}"))?;
    stream
        .write_all(&(body.len() as u32).to_be_bytes())
        .await
        .map_err(|e| format!("发送请求体失败: {e}"))?;
    if !body.is_empty() {
        stream
            .write_all(body)
            .await
            .map_err(|e| format!("发送请求体失败: {e}"))?;
    }
    stream
        .flush()
        .await
        .map_err(|e| format!("发送请求失败: {e}"))?;

    let mut status_buf = [0u8; 1];
    stream
        .read_exact(&mut status_buf)
        .await
        .map_err(|e| format!("读取响应状态失败: {e}"))?;
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| format!("读取响应长度失败: {e}"))?;
    let data_len = u32::from_be_bytes(len_buf) as usize;
    if data_len > 1024 * 1024 {
        return Err("响应体过大".to_string());
    }
    let mut data = vec![0u8; data_len];
    if data_len > 0 {
        stream
            .read_exact(&mut data)
            .await
            .map_err(|e| format!("读取响应体失败: {e}"))?;
    }
    Ok((status_buf[0], data))
}

/// 解析联机中心地址：显式参数优先，否则经 easytier-cli 从虚拟网络自动发现。
///
/// 返回 (center_ip, center_port)。hint 全部提供时直接返回；任一缺失时调用
/// `EasyTier::discover_center`（按 hostname 前缀 `scaffolding-mc-server-` 匹配房主节点）。
pub async fn resolve_center_addr(
    center_ip_hint: Option<&str>,
    center_port_hint: Option<u16>,
    easytier: &EasyTier,
) -> Result<(String, u16), String> {
    match (center_ip_hint, center_port_hint) {
        (Some(ip), Some(port)) => Ok((ip.to_string(), port)),
        _ => easytier.discover_center().await,
    }
}

/// 房客发现流程：校验中心连通性、协商协议列表、获取 MC 服务器端口。
///
/// 返回 (mc_ip, mc_port)，mc_ip 即联机中心虚拟 IP。
pub async fn discover_mc(center_ip: &str, center_port: u16) -> Result<(String, u16), String> {
    let connect = timeout(
        REQUEST_TIMEOUT,
        TcpStream::connect((center_ip, center_port)),
    )
    .await
    .map_err(|_| format!("连接联机中心 {center_ip}:{center_port} 超时"))?
    .map_err(|e| format!("连接联机中心 {center_ip}:{center_port} 失败: {e}"))?;

    let mut stream = connect;
    let _ = stream.set_nodelay(true);

    // c:ping：指纹回显校验
    let (status, data) = timeout(
        REQUEST_TIMEOUT,
        send_request(&mut stream, "c:ping", FINGERPRINT),
    )
    .await
    .map_err(|_| "c:ping 超时".to_string())??;
    if status != 0 || data.as_slice() != FINGERPRINT {
        return Err("联机中心指纹校验失败".to_string());
    }

    // c:protocols：确认标准协议
    let (status, data) = timeout(
        REQUEST_TIMEOUT,
        send_request(&mut stream, "c:protocols", &[]),
    )
    .await
    .map_err(|_| "c:protocols 超时".to_string())??;
    if status != 0 {
        return Err(format!("c:protocols 失败，状态 {status}"));
    }
    let protocols = String::from_utf8_lossy(&data);
    if !protocols.split('\0').any(|p| p == "c:server_port") {
        return Err("联机中心不支持 c:server_port 协议".to_string());
    }

    // c:server_port：获取 MC 端口
    let (status, data) = timeout(
        REQUEST_TIMEOUT,
        send_request(&mut stream, "c:server_port", &[]),
    )
    .await
    .map_err(|_| "c:server_port 超时".to_string())??;
    if status == 32 {
        return Err("MC 服务器尚未启动".to_string());
    }
    if status != 0 || data.len() != 2 {
        return Err(format!("c:server_port 失败，状态 {status}"));
    }
    let mc_port = u16::from_be_bytes([data[0], data[1]]);
    Ok((center_ip.to_string(), mc_port))
}
