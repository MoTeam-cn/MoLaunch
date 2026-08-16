//! 地址延迟测试纯函数单测（parse_ping_rtt）

use super::parse_ping_rtt;

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
