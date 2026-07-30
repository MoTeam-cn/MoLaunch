//! protocol 单元测试

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
fn test_turn_servers_roundtrip() {
    // 模拟 IceServerEntry[] 的 JSON 序列化字节
    let json = br#"[{"urls":["turn:turn.example.com:3478"],"username":"foo","credential":"bar"}]"#;
    let original = turn_servers_message(7, json);
    let encoded = encode(&original);
    // type(1) + seq(4) + length(2) + subtype(1) + json(N) = 8 + N
    assert_eq!(encoded.len(), 8 + json.len());

    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, original);

    // 验证 payload 字节与原 JSON 一致
    match decoded {
        Message::Control { subtype, payload, .. } => {
            assert_eq!(subtype, ControlSubtype::TurnServers);
            assert_eq!(payload.as_slice(), json);
        }
        _ => panic!("期望 Control 消息"),
    }
}

#[test]
fn test_turn_servers_empty_list() {
    // 空列表也应能编码/解码（参与者收到后跳过，不重建 PC）
    let empty_json = b"[]";
    let original = turn_servers_message(1, empty_json);
    let encoded = encode(&original);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, original);
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
