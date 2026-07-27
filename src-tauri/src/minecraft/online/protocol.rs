//! DataChannel 消息协议（阶段三子任务 2）
//!
//! 定义 P2P DataChannel 上传输的二进制消息帧格式，用于虚拟网卡 IP 包转发
//! 与控制消息（心跳、状态查询）。
//!
//! # 帧格式
//!
//! 所有消息均为二进制，大端序（网络字节序）：
//!
//! ```text
//! +--------+--------+--------+--------+--------+--------+-----------+
//! | type   |    seq (u32 BE)    |    length (u16 BE)  | payload     |
//! | 1 byte |       4 bytes      |       2 bytes       | N bytes     |
//! +--------+--------+--------+--------+--------+--------+-----------+
//! ```
//!
//! - `type`：消息类型（1 字节）
//!   - `0x01` = Data（IP 包）
//!   - `0x02` = Control（控制消息）
//!   - `0x03` = Error（错误消息）
//! - `seq`：序列号（4 字节，u32 大端序），用于检测丢包与乱序
//! - `length`：payload 长度（2 字节，u16 大端序，最大 65535）
//! - `payload`：消息体
//!   - Data：原始 IP 包字节
//!   - Control：`[subtype: u8, ...]`
//!   - Error：UTF-8 错误描述字符串
//!
//! # Control 子类型
//!
//! - `0x01` = Heartbeat（心跳，payload 仅 1 字节 subtype）
//! - `0x02` = StatusQuery（状态查询，payload 仅 1 字节 subtype）
//! - `0x03` = StatusResponse（状态响应，payload = subtype + 状态 JSON）
//! - `0x04` = HostMcPort（房主 MC 局域网端口，payload = subtype + 2 字节大端序 u16 端口）
//!
//! # 设计决策
//!
//! - **二进制而非 JSON**：IP 包是二进制数据，JSON 编码会有 base64 开销（+33% 体积）
//! - **大端序**：网络字节序，跨平台一致
//! - **u16 length**：DataChannel 单条消息最大 16KB（浏览器限制），u16 足够
//! - **seq 字段**：DataChannel 配置 `maxRetransmits: 0`（UDP 语义），seq 用于检测丢包

use std::io::{self, Cursor, Read};

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// 数据消息（IP 包）
    Data = 0x01,
    /// 控制消息（心跳、状态查询）
    Control = 0x02,
    /// 错误消息
    Error = 0x03,
}

impl MessageType {
    /// 从 u8 解析消息类型
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x01 => Some(Self::Data),
            0x02 => Some(Self::Control),
            0x03 => Some(Self::Error),
            _ => None,
        }
    }
}

/// 控制消息子类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ControlSubtype {
    /// 心跳
    Heartbeat = 0x01,
    /// 状态查询
    StatusQuery = 0x02,
    /// 状态响应
    StatusResponse = 0x03,
    /// 房主 MC 局域网端口（房主开放 LAN 后广播给所有参与者）
    HostMcPort = 0x04,
}

impl ControlSubtype {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x01 => Some(Self::Heartbeat),
            0x02 => Some(Self::StatusQuery),
            0x03 => Some(Self::StatusResponse),
            0x04 => Some(Self::HostMcPort),
            _ => None,
        }
    }
}

/// 消息帧（解析后的抽象表示）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// 数据消息（IP 包）
    Data { seq: u32, payload: Vec<u8> },
    /// 控制消息
    Control { seq: u32, subtype: ControlSubtype, payload: Vec<u8> },
    /// 错误消息
    Error { seq: u32, message: String },
}

/// 帧头部长度（type + seq + length = 1 + 4 + 2 = 7 字节）
pub const FRAME_HEADER_LEN: usize = 7;

/// 最大 payload 长度（u16::MAX）
pub const MAX_PAYLOAD_LEN: usize = 65535;

/// 序列化消息帧为二进制字节
///
/// 返回 `Vec<u8>`，可直接通过 `RTCDataChannel.send(ArrayBuffer)` 发送。
pub fn encode(msg: &Message) -> Vec<u8> {
    let (msg_type, seq, payload): (u8, u32, Vec<u8>) = match msg {
        Message::Data { seq, payload } => (MessageType::Data as u8, *seq, payload.clone()),
        Message::Control { seq, subtype, payload } => {
            // Control 消息 payload 前追加 subtype 字节
            let mut full_payload = vec![*subtype as u8];
            full_payload.extend_from_slice(payload);
            (MessageType::Control as u8, *seq, full_payload)
        }
        Message::Error { seq, message } => {
            (MessageType::Error as u8, *seq, message.as_bytes().to_vec())
        }
    };

    let length = payload.len() as u16;
    let mut buf = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());
    buf.push(msg_type);
    buf.extend_from_slice(&seq.to_be_bytes());
    buf.extend_from_slice(&length.to_be_bytes());
    buf.extend_from_slice(&payload);
    buf
}

/// 从字节流反序列化消息帧
///
/// 期望 `bytes` 包含完整的一帧（头部 + payload）。
/// 若 payload 长度不足，返回 `UnexpectedEof`。
pub fn decode(bytes: &[u8]) -> io::Result<Message> {
    if bytes.len() < FRAME_HEADER_LEN {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("帧头部不完整：期望 {} 字节，实际 {}", FRAME_HEADER_LEN, bytes.len()),
        ));
    }

    let mut cursor = Cursor::new(bytes);
    let mut type_buf = [0u8; 1];
    cursor.read_exact(&mut type_buf)?;
    let msg_type = MessageType::from_u8(type_buf[0]).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("未知消息类型: 0x{:02x}", type_buf[0]),
        )
    })?;

    let mut seq_buf = [0u8; 4];
    cursor.read_exact(&mut seq_buf)?;
    let seq = u32::from_be_bytes(seq_buf);

    let mut length_buf = [0u8; 2];
    cursor.read_exact(&mut length_buf)?;
    let length = u16::from_be_bytes(length_buf) as usize;

    let mut payload = vec![0u8; length];
    cursor.read_exact(&mut payload)?;

    Ok(match msg_type {
        MessageType::Data => Message::Data { seq, payload },
        MessageType::Control => {
            if payload.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Control 消息 payload 不能为空（至少需 1 字节 subtype）",
                ));
            }
            let subtype = ControlSubtype::from_u8(payload[0]).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("未知控制子类型: 0x{:02x}", payload[0]),
                )
            })?;
            Message::Control {
                seq,
                subtype,
                payload: payload[1..].to_vec(),
            }
        }
        MessageType::Error => {
            let message = String::from_utf8(payload).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("错误消息 UTF-8 解码失败: {}", e))
            })?;
            Message::Error { seq, message }
        }
    })
}

/// 便捷构造：数据消息（IP 包）
pub fn data_message(seq: u32, ip_packet: &[u8]) -> Message {
    Message::Data { seq, payload: ip_packet.to_vec() }
}

/// 便捷构造：心跳控制消息
pub fn heartbeat_message(seq: u32) -> Message {
    Message::Control { seq, subtype: ControlSubtype::Heartbeat, payload: vec![] }
}

/// 便捷构造：状态查询控制消息
pub fn status_query_message(seq: u32) -> Message {
    Message::Control { seq, subtype: ControlSubtype::StatusQuery, payload: vec![] }
}

/// 便捷构造：状态响应控制消息
pub fn status_response_message(seq: u32, status_json: &[u8]) -> Message {
    Message::Control {
        seq,
        subtype: ControlSubtype::StatusResponse,
        payload: status_json.to_vec(),
    }
}

/// 便捷构造：房主 MC 局域网端口控制消息
///
/// 房主开放 LAN 后，通过 DataChannel 广播此消息通知所有参与者。
/// payload 编码：2 字节大端序 u16 端口号。
pub fn host_mc_port_message(seq: u32, port: u16) -> Message {
    Message::Control {
        seq,
        subtype: ControlSubtype::HostMcPort,
        payload: port.to_be_bytes().to_vec(),
    }
}

/// 解析 HostMcPort 控制消息的 payload 为端口号
///
/// 期望 payload 长度为 2 字节（大端序 u16）。其他长度返回 None。
pub fn parse_host_mc_port_payload(payload: &[u8]) -> Option<u16> {
    if payload.len() != 2 {
        return None;
    }
    Some(u16::from_be_bytes([payload[0], payload[1]]))
}

/// 便捷构造：错误消息
pub fn error_message(seq: u32, message: &str) -> Message {
    Message::Error { seq, message: message.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_message_roundtrip() {
        let original = data_message(42, &[0x45, 0x00, 0x00, 0x28]); // IP 头前 4 字节
        let encoded = encode(&original);
        assert_eq!(encoded.len(), FRAME_HEADER_LEN + 4);

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_heartbeat_message_roundtrip() {
        let original = heartbeat_message(1);
        let encoded = encode(&original);
        // type(1) + seq(4) + length(2) + subtype(1) = 8 字节
        assert_eq!(encoded.len(), 8);

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_error_message_roundtrip() {
        let original = error_message(99, "TUN 接口读取失败");
        let encoded = encode(&original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_status_response_roundtrip() {
        let status = br#"{"tun":"up","peers":2}"#;
        let original = status_response_message(7, status);
        let encoded = encode(&original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_host_mc_port_roundtrip() {
        let port: u16 = 49152;
        let original = host_mc_port_message(3, port);
        let encoded = encode(&original);
        // type(1) + seq(4) + length(2) + subtype(1) + port(2) = 10 字节
        assert_eq!(encoded.len(), 10);

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);

        // 验证 payload 解析
        match decoded {
            Message::Control { payload, subtype, .. } => {
                assert_eq!(subtype, ControlSubtype::HostMcPort);
                assert_eq!(parse_host_mc_port_payload(&payload), Some(port));
            }
            _ => panic!("期望 Control 消息"),
        }
    }

    #[test]
    fn test_parse_host_mc_port_invalid_payload() {
        // 长度不为 2 的 payload 应返回 None
        assert_eq!(parse_host_mc_port_payload(&[]), None);
        assert_eq!(parse_host_mc_port_payload(&[0x01]), None);
        assert_eq!(parse_host_mc_port_payload(&[0x01, 0x02, 0x03]), None);
    }

    #[test]
    fn test_decode_invalid_type() {
        // 构造无效 type
        let bytes = [0xFF, 0, 0, 0, 1, 0, 0]; // type=0xFF, seq=1, length=0
        let result = decode(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未知消息类型"));
    }

    #[test]
    fn test_decode_short_header() {
        let bytes = [0x01, 0, 0]; // 仅 3 字节，不够 7 字节头部
        let result = decode(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().kind() == io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_decode_control_empty_payload() {
        // Control 消息但 payload 为空（length=0）
        let bytes = [0x02, 0, 0, 0, 1, 0, 0]; // type=0x02, seq=1, length=0
        let result = decode(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("payload 不能为空"));
    }
}
