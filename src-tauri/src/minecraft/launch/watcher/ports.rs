//! 游戏进程监听端口扫描
//!
//! 供 watcher 端口轮询与联机模块进房回查共用（MC 局域网端口识别）。

/// 枚举指定进程监听的 TCP 端口（排除回环地址）
///
/// 基于 netstat2 直接读取系统套接字表，不依赖游戏日志格式与 stdout 可用性；
/// MC 开放局域网后由 Java 进程监听一个非回环 TCP 端口，据此自动识别上报。
pub(crate) fn listening_tcp_ports(pid: u32) -> Vec<u16> {
    let af_flags = netstat2::AddressFamilyFlags::all();
    let proto_flags = netstat2::ProtocolFlags::TCP;
    let Ok(sockets) = netstat2::get_sockets_info(af_flags, proto_flags) else {
        return Vec::new();
    };
    let mut ports = Vec::new();
    for sock in sockets {
        if !sock.associated_pids.contains(&pid) {
            continue;
        }
        if let netstat2::ProtocolSocketInfo::Tcp(tcp) = sock.protocol_socket_info {
            if tcp.state != netstat2::TcpState::Listen {
                continue;
            }
            // 回环监听多为 JVM 内部服务（RMI 等），排除以降低误报
            if tcp.local_addr.is_loopback() {
                continue;
            }
            ports.push(tcp.local_port);
        }
    }
    ports.sort_unstable();
    ports.dedup();
    ports
}

#[cfg(test)]
mod tests {
    use super::listening_tcp_ports;

    #[test]
    fn nonexistent_pid_returns_empty() {
        assert!(listening_tcp_ports(u32::MAX).is_empty());
    }
}
