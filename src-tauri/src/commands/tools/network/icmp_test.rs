//! icmp 模块纯函数单测（checksum / 报文构造 / 回包解析）

use socket2::Type as SockType;

use super::{checksum, make_echo_request, parse_echo_reply};

#[test]
fn checksum_recomputed_is_zero() {
    // 发送端：校验和字段（偏移 [2..4]）先置 0，对整个报文求和取反后写入；
    // 接收端：对含校验和的整个报文再求和取反必为 0
    let mut data: [u8; 4] = [0x45, 0x00, 0x00, 0x73];
    data[2..4].copy_from_slice(&[0, 0]);
    let sum = checksum(&data);
    let mut with_sum = data.to_vec();
    with_sum[2..4].copy_from_slice(&sum.to_be_bytes());
    assert_eq!(checksum(&with_sum), 0);
}

#[test]
fn make_echo_request_raw() {
    let pkt = make_echo_request(0x1234, SockType::RAW, &[1, 2, 3]);
    assert_eq!(pkt[0], 8);
    assert_eq!(pkt[1], 0);
    assert_eq!(&pkt[4..6], &[0x12, 0x34]);
    assert_eq!(&pkt[6..8], &[0, 0]);
    assert_eq!(&pkt[8..], &[1, 2, 3]);
    // RAW 下校验和由本端计算，非零且可复原为 0
    assert_ne!(&pkt[2..4], &[0, 0]);
    assert_eq!(checksum(&pkt), 0);
}

#[test]
fn parse_echo_reply_ipv4_packet() {
    // 构造 RAW 收到的完整 IPv4 包：V4 + 头长 20 + 总长 28 + ICMP Echo Reply（id=0x1234 seq=0）
    let mut ip = [0u8; 20];
    ip[0] = 0x45;
    ip[2..4].copy_from_slice(&28u16.to_be_bytes());
    let mut icmp = [0u8; 8];
    icmp[0] = 0;
    icmp[4..6].copy_from_slice(&0x1234u16.to_be_bytes());
    let mut pkt = Vec::new();
    pkt.extend_from_slice(&ip);
    pkt.extend_from_slice(&icmp);
    assert_eq!(parse_echo_reply(&pkt, SockType::RAW), Some((0x1234, 0)));
}

#[test]
fn parse_echo_reply_rejects_non_reply() {
    // IPv4 包内嵌 ICMP Echo Request（type=8），非 Reply 应被拒绝
    let mut ip = [0u8; 20];
    ip[0] = 0x45;
    ip[2..4].copy_from_slice(&28u16.to_be_bytes());
    let mut icmp = [0u8; 8];
    icmp[0] = 8;
    let mut pkt = Vec::new();
    pkt.extend_from_slice(&ip);
    pkt.extend_from_slice(&icmp);
    assert_eq!(parse_echo_reply(&pkt, SockType::RAW), None);
}
