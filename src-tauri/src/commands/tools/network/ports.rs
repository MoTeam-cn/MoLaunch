use sysinfo::{PidExt, ProcessExt, SystemExt};

use crate::log_info;
use crate::state::AppState;

use super::super::types::{ListOpenPortsResult, OpenPortInfo};

/// 同步获取本机监听端口列表（供 picker URI scheme handler 调用）
///
/// 与 `list_open_ports` 的区别：不依赖 AppState，直接同步枚举端口。
/// `list_open_ports` 调用本函数后序列化，避免逻辑重复。
/// 枚举失败时返回空列表（picker 场景下比抛错更友好）。
pub fn list_open_ports_sync() -> Vec<OpenPortInfo> {
    // 同时枚举 IPv4/IPv6 的 TCP + UDP 套接字
    let af_flags = netstat2::AddressFamilyFlags::all();
    let proto_flags = netstat2::ProtocolFlags::TCP | netstat2::ProtocolFlags::UDP;
    let sockets = match netstat2::get_sockets_info(af_flags, proto_flags) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let sys = sysinfo::System::new_all();

    let mut ports: Vec<OpenPortInfo> = Vec::new();

    for sock in sockets {
        // 解构出协议信息与关联 PID 列表（Linux 上还有 inode/uid 字段，用 .. 忽略）
        let netstat2::SocketInfo {
            protocol_socket_info,
            associated_pids,
            ..
        } = sock;

        match protocol_socket_info {
            netstat2::ProtocolSocketInfo::Tcp(tcp) => {
                // 仅保留 LISTEN 状态的 TCP 套接字
                if tcp.state != netstat2::TcpState::Listen {
                    continue;
                }
                let pid = associated_pids.first().copied();
                let process_name = pid.and_then(|p| {
                    sys.process(sysinfo::Pid::from_u32(p))
                        .map(|proc_| proc_.name().to_string())
                });
                ports.push(OpenPortInfo {
                    local_addr: format!("{}:{}", tcp.local_addr, tcp.local_port),
                    port: tcp.local_port,
                    protocol: "tcp".to_string(),
                    process_name,
                    pid,
                });
            }
            netstat2::ProtocolSocketInfo::Udp(udp) => {
                // UDP 无连接状态，全部视为监听
                let pid = associated_pids.first().copied();
                let process_name = pid.and_then(|p| {
                    sys.process(sysinfo::Pid::from_u32(p))
                        .map(|proc_| proc_.name().to_string())
                });
                ports.push(OpenPortInfo {
                    local_addr: format!("{}:{}", udp.local_addr, udp.local_port),
                    port: udp.local_port,
                    protocol: "udp".to_string(),
                    process_name,
                    pid,
                });
            }
        }
    }

    // 按 port 升序排序（同 port 内按 protocol / local_addr 二级排序以便去重）
    ports.sort_by(|a, b| {
        a.port
            .cmp(&b.port)
            .then(a.protocol.cmp(&b.protocol))
            .then(a.local_addr.cmp(&b.local_addr))
    });
    // 去重：同一 (port, protocol, local_addr) 可能被枚举多次
    ports.dedup_by(|a, b| {
        a.port == b.port && a.protocol == b.protocol && a.local_addr == b.local_addr
    });

    ports
}

/// 列出本机所有监听中的 TCP/UDP 端口
///
/// 用于 Frp 创建隧道时选择内网端口：枚举本机所有处于 LISTEN 状态的 TCP 端口
/// 与所有 UDP 端口（UDP 无连接状态，全部视为监听），并尝试通过 sysinfo 解析占用进程。
/// 按 port 升序排序，去重同一 (port, protocol, local_addr) 的重复条目。
pub async fn list_open_ports(state: &AppState) -> Result<serde_json::Value, String> {
    let _game_dir = {
        let config = state.config.lock().await;
        crate::state::resolve_game_dir(&config.game_dir)
    };

    let ports = list_open_ports_sync();
    log_info!("[ListOpenPorts] 找到 {} 个监听端口", ports.len());

    let result = ListOpenPortsResult { ports };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}
