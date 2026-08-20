//! 自实现单次 ICMPv4 Echo ping
//!
//! socket 方案参考 surge-ping：优先 Linux 非特权 ICMP（SOCK_DGRAM），失败回退
//! SOCK_RAW；报文构造、校验和与回包解析自实现，不依赖系统 ping 命令。

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type as SockType};
use tokio::net::UdpSocket;

/// ICMP Echo Request 报文类型
const ECHO_REQUEST: u8 = 8;
/// ICMP Echo Reply 报文类型
const ECHO_REPLY: u8 = 0;
/// ICMP Echo 报文头长度（type + code + checksum + identifier + sequence）
const ECHO_HEADER_LEN: usize = 8;

/// 单次 ICMPv4 Echo ping，成功返回 RTT
pub async fn ping_once(ip: Ipv4Addr, timeout: Duration) -> Result<Duration, String> {
    let (sock_type, udp) = create_icmp_socket()?;
    // identifier 取进程号低位；sequence 固定 0（单次探测）
    let ident = std::process::id() as u16;
    let packet = make_echo_request(ident, sock_type, &[0u8; 8]);
    let target = SocketAddr::new(IpAddr::V4(ip), 0);
    udp.send_to(&packet, target)
        .await
        .map_err(|e| format!("发送 ICMP Echo 失败: {e}"))?;
    let start = Instant::now();
    let mut buf = [0u8; 2048];
    loop {
        let remain = timeout.saturating_sub(start.elapsed());
        if remain.is_zero() {
            return Err("ping 超时（主机不可达或禁 ICMP）".to_string());
        }
        let (size, from) = match tokio::time::timeout(remain, udp.recv_from(&mut buf)).await {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => return Err(format!("接收 ICMP 回包失败: {e}")),
            Err(_) => return Err("ping 超时（主机不可达或禁 ICMP）".to_string()),
        };
        if from.ip() != IpAddr::V4(ip) {
            continue;
        }
        if let Some((r_ident, r_seq)) = parse_echo_reply(&buf[..size], sock_type) {
            // Linux 非特权 ICMP（DGRAM）下 identifier 由内核分配，仅按序号匹配
            if r_seq == 0 && (is_linux_icmp_socket(sock_type) || r_ident == ident) {
                return Ok(start.elapsed());
            }
        }
    }
}

/// 创建 ICMPv4 socket，返回 (socket 类型, tokio UdpSocket)
///
/// 优先 SOCK_DGRAM（Linux 非特权 ICMP datagram socket），失败回退 SOCK_RAW
/// （Windows / Unix 需要 CAP_NET_RAW 或 root 时同样可创建）。
fn create_icmp_socket() -> Result<(SockType, UdpSocket), String> {
    let (sock_type, socket) = match make_icmp_socket(SockType::DGRAM) {
        Ok(s) => (SockType::DGRAM, s),
        Err(dgram_err) => match make_icmp_socket(SockType::RAW) {
            Ok(s) => (SockType::RAW, s),
            Err(raw_err) => {
                return Err(format!(
                    "创建 ICMP socket 失败（DGRAM: {dgram_err}；RAW: {raw_err}）"
                ))
            }
        },
    };
    socket
        .set_nonblocking(true)
        .map_err(|e| format!("设置非阻塞失败: {e}"))?;
    let udp =
        UdpSocket::from_std(socket.into()).map_err(|e| format!("包装 ICMP socket 失败: {e}"))?;
    Ok((sock_type, udp))
}

fn make_icmp_socket(sock_type: SockType) -> io::Result<Socket> {
    Socket::new(Domain::IPV4, sock_type, Some(Protocol::ICMPV4))
}

/// 判断是否为 Linux 非特权 ICMP socket（SOCK_DGRAM：内核填充 identifier 与校验和）
fn is_linux_icmp_socket(sock_type: SockType) -> bool {
    sock_type == SockType::DGRAM && cfg!(target_os = "linux")
}

/// 构造 ICMP Echo Request 报文（8 字节头 + payload）
fn make_echo_request(ident: u16, sock_type: SockType, payload: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; ECHO_HEADER_LEN + payload.len()];
    buf[0] = ECHO_REQUEST;
    buf[1] = 0;
    buf[4..6].copy_from_slice(&ident.to_be_bytes());
    // sequence = 0（单次探测）
    buf[6..8].copy_from_slice(&0u16.to_be_bytes());
    buf[ECHO_HEADER_LEN..].copy_from_slice(payload);
    if !is_linux_icmp_socket(sock_type) {
        // RAW socket：identifier 与校验和由本端计算（checksum 字段先置 0，再对整个报文求和取反）
        let sum = checksum(&buf);
        buf[2..4].copy_from_slice(&sum.to_be_bytes());
    }
    buf
}

/// 解析回包是否为匹配的 Echo Reply，返回 (identifier, sequence)
fn parse_echo_reply(buf: &[u8], sock_type: SockType) -> Option<(u16, u16)> {
    let icmp = if is_linux_icmp_socket(sock_type) {
        // DGRAM：内核已剥离 IPv4 头，直接是 ICMP 报文
        buf
    } else {
        // RAW：收到完整 IPv4 包，剥离 IP 头后取 ICMP 部分
        if buf.len() < 20 || (buf[0] >> 4) != 4 {
            return None;
        }
        let ihl = usize::from(buf[0] & 0x0f) * 4;
        let total = usize::from(u16::from_be_bytes([buf[2], buf[3]]));
        if ihl < 20 || total < ihl + ECHO_HEADER_LEN || buf.len() < total {
            return None;
        }
        &buf[ihl..total]
    };
    if icmp.len() < ECHO_HEADER_LEN || icmp[0] != ECHO_REPLY || icmp[1] != 0 {
        return None;
    }
    Some((
        u16::from_be_bytes([icmp[4], icmp[5]]),
        u16::from_be_bytes([icmp[6], icmp[7]]),
    ))
}

/// Internet checksum（RFC 1071），返回补码（发送端先置 0 再对报文求和取反）
fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let (pairs, remainder) = data.as_chunks::<2>();
    for pair in pairs {
        sum += u32::from(u16::from_be_bytes([pair[0], pair[1]]));
    }
    if let Some(&b) = remainder.first() {
        sum += u32::from(b) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

#[cfg(test)]
#[path = "icmp_test.rs"]
mod tests;
