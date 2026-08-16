//! 地址延迟测试纯函数单测（parse_ping_rtt / decode_output）

use super::{decode_output, parse_ping_rtt};

#[test]
fn parse_ping_rtt_english() {
    assert_eq!(
        parse_ping_rtt("Reply from 1.2.3.4: bytes=32 time=12ms TTL=52"),
        Some(12)
    );
    assert_eq!(parse_ping_rtt("time=12.3 ms"), Some(12));
    assert_eq!(parse_ping_rtt("time<1ms TTL=52"), Some(1));
    assert_eq!(parse_ping_rtt("time=1234ms"), Some(1234));
    assert_eq!(
        parse_ping_rtt("PING 1.2.3.4 (1.2.3.4) 56(84) bytes of data."),
        None
    );
}

#[test]
fn parse_ping_rtt_chinese() {
    assert_eq!(
        parse_ping_rtt("来自 1.2.3.4 的回复: 字节=32 时间=12ms TTL=52"),
        Some(12)
    );
    assert_eq!(parse_ping_rtt("时间<1ms"), Some(1));
}

#[test]
fn decode_output_gbk_ping() {
    // 中文 Windows 默认代码页 936（GBK）的 ping 输出：
    // "来自 1.2.3.4 的回复: 字节=32 时间=12ms TTL=52"
    let gbk = [
        0xC0, 0xB4, 0xD7, 0xD4, 0x20, 0x31, 0x2E, 0x32, 0x2E, 0x33, 0x2E, 0x34, 0x20, 0xB5, 0xC4,
        0xBB, 0xD8, 0xB8, 0xB4, 0x3A, 0x20, 0xD7, 0xD6, 0xBD, 0xDA, 0x3D, 0x33, 0x32, 0x20, 0xCA,
        0xB1, 0xBC, 0xE4, 0x3D, 0x31, 0x32, 0x6D, 0x73, 0x20, 0x54, 0x54, 0x4C, 0x3D, 0x35, 0x32,
    ];
    let text = decode_output(&gbk);
    assert_eq!(text, "来自 1.2.3.4 的回复: 字节=32 时间=12ms TTL=52");
    assert_eq!(parse_ping_rtt(&text), Some(12));
}

#[test]
fn decode_output_utf8_passthrough() {
    assert_eq!(decode_output(b"time=12ms TTL=52"), "time=12ms TTL=52");
}
