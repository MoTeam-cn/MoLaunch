//! DoH 单测：JSON 解析、记录过滤、熔断状态

use super::{
    breaker_open, note_failure, note_success, parse_answers, BREAKER_OPEN, BREAKER_THRESHOLD,
};
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};

/// 腾讯风格响应：CNAME + A 混排
const SAMPLE: &str = r#"{"Status":0,"TC":false,"Answer":[
{"name":"cdn.modrinth.com.","type":5,"TTL":60,"data":"xyz.atlas.fastly.net."},
{"name":"xyz.atlas.fastly.net.","type":1,"TTL":30,"data":"151.101.1.69"},
{"name":"xyz.atlas.fastly.net.","type":1,"TTL":30,"data":"151.101.65.69"}
]}"#;

#[test]
fn test_parse_answers_只取A与AAAA记录() {
    let addrs = parse_answers(SAMPLE).expect("应解析成功");
    assert_eq!(
        addrs,
        vec![
            IpAddr::V4(Ipv4Addr::new(151, 101, 1, 69)),
            IpAddr::V4(Ipv4Addr::new(151, 101, 65, 69)),
        ]
    );
}

#[test]
fn test_parse_answers_无Answer字段返回空() {
    let addrs = parse_answers(r#"{"Status":3}"#).expect("无 Answer 不算错误");
    assert!(addrs.is_empty());
}

#[test]
fn test_parse_answers_非法JSON报错() {
    assert!(parse_answers("not json").is_err());
}

#[test]
fn test_parse_answers_过滤私有地址与不可解析数据() {
    let body = r#"{"Answer":[
{"type":1,"data":"192.168.1.1"},
{"type":1,"data":"127.0.0.1"},
{"type":1,"data":"198.18.0.9"},
{"type":1,"data":"not-an-ip"},
{"type":28,"data":"2400:3200::1"},
{"type":1,"data":"8.8.8.8"}
]}"#;
    let addrs = parse_answers(body).expect("应解析成功");
    assert_eq!(addrs.len(), 2, "只应保留公网 IPv4 与公网 IPv6");
    assert!(addrs.contains(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
}

#[test]
fn test_parse_answers_重复地址去重() {
    let body = r#"{"Answer":[
{"type":1,"data":"8.8.8.8"},
{"type":1,"data":"8.8.8.8"}
]}"#;
    assert_eq!(parse_answers(body).expect("应解析成功").len(), 1);
}

#[test]
fn test_breaker_成功清零失败计数() {
    let now = Instant::now();
    note_failure(now);
    note_success();
    assert!(!breaker_open(now));
}

#[test]
fn test_breaker_连续失败达到阈值后打开并在超时后恢复() {
    let now = Instant::now();
    note_success();
    for _ in 0..BREAKER_THRESHOLD {
        note_failure(now);
    }
    assert!(breaker_open(now), "达到阈值后应熔断");
    assert!(
        breaker_open(now + BREAKER_OPEN - Duration::from_secs(1)),
        "熔断期内仍应打开"
    );
    assert!(!breaker_open(now + BREAKER_OPEN), "熔断到期应恢复");
    note_success();
}
