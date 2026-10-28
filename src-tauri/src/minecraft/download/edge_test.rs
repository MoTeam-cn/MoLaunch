//! 边缘节点优选单测（只测纯逻辑：地址过滤、切换门槛、预算窗口、名单匹配）

use super::{
    eligible_host, is_fresh, should_switch, usable_candidate, ProbeBudget, BUDGET_WINDOW,
    CHOICE_TTL, PROBE_BUDGET_BYTES, SWITCH_GAIN,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{Duration, Instant};

fn v4(ip: [u8; 4]) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]))
}

#[test]
fn test_usable_candidate_公网地址可用() {
    assert!(usable_candidate(v4([8, 8, 8, 8])));
    assert!(usable_candidate(v4([151, 101, 1, 69])));
    assert!(usable_candidate(IpAddr::V6(
        "2400:3200::1".parse::<Ipv6Addr>().expect("合法 IPv6")
    )));
}

#[test]
fn test_usable_candidate_内网与保留段排除() {
    assert!(!usable_candidate(v4([127, 0, 0, 1])));
    assert!(!usable_candidate(v4([192, 168, 1, 1])));
    assert!(!usable_candidate(v4([10, 0, 0, 1])));
    assert!(!usable_candidate(v4([172, 16, 0, 1])));
    assert!(!usable_candidate(v4([169, 254, 1, 1])));
    assert!(!usable_candidate(v4([224, 0, 0, 1])));
    assert!(!usable_candidate(v4([0, 0, 0, 0])));
}

#[test]
fn test_usable_candidate_代理假地址与CGNAT排除() {
    // 198.18.0.0/15：TUN 代理给本机的假地址，测速结果全在本地回环
    assert!(!usable_candidate(v4([198, 18, 0, 5])));
    assert!(!usable_candidate(v4([198, 19, 255, 254])));
    // 100.64.0.0/10：运营商级 NAT
    assert!(!usable_candidate(v4([100, 64, 0, 1])));
    assert!(!usable_candidate(v4([100, 127, 255, 254])));
    // 边界外仍可用
    assert!(usable_candidate(v4([100, 63, 0, 1])));
    assert!(usable_candidate(v4([100, 128, 0, 1])));
    assert!(usable_candidate(v4([198, 20, 0, 1])));
}

#[test]
fn test_usable_candidate_ipv6特殊段排除() {
    assert!(!usable_candidate(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert!(!usable_candidate(IpAddr::V6(Ipv6Addr::UNSPECIFIED)));
    assert!(!usable_candidate(IpAddr::V6(
        "fc00::1".parse::<Ipv6Addr>().expect("合法 IPv6")
    )));
    assert!(!usable_candidate(IpAddr::V6(
        "fe80::1".parse::<Ipv6Addr>().expect("合法 IPv6")
    )));
    assert!(!usable_candidate(IpAddr::V6(
        "ff02::1".parse::<Ipv6Addr>().expect("合法 IPv6")
    )));
}

#[test]
fn test_usable_candidate_映射地址按IPv4判断() {
    assert!(usable_candidate(IpAddr::V6(
        "::ffff:8.8.8.8".parse::<Ipv6Addr>().expect("合法 IPv6")
    )));
    assert!(!usable_candidate(IpAddr::V6(
        "::ffff:192.168.1.1".parse::<Ipv6Addr>().expect("合法 IPv6")
    )));
}

#[test]
fn test_should_switch_没有当前节点直接采用() {
    assert!(should_switch(1, None));
}

#[test]
fn test_should_switch_收益不足不切换() {
    // 门槛 25%：刚好 25% 不算「明显更快」
    assert!(!should_switch(125, Some(100)));
    assert!(!should_switch(100, Some(100)));
    assert!(!should_switch(90, Some(100)));
    assert!(should_switch(126, Some(100)));
    // 门槛值与实现同步，避免改了常量忘了改测试
    let just_over = (100.0 * (1.0 + SWITCH_GAIN)) as u64 + 1;
    assert!(should_switch(just_over, Some(100)));
}

#[test]
fn test_is_fresh_有效期内为真() {
    let at = Instant::now();
    assert!(is_fresh(at, at));
    assert!(is_fresh(at, at + CHOICE_TTL - Duration::from_millis(1)));
    assert!(!is_fresh(at, at + CHOICE_TTL));
}

#[test]
fn test_is_fresh_时间倒退按过期处理() {
    let now = Instant::now();
    assert!(!is_fresh(now + Duration::from_secs(5), now));
}

#[test]
fn test_eligible_host_命中名单与子域() {
    assert!(eligible_host("cdn.modrinth.com"));
    assert!(eligible_host("cdn.curseforge.com"));
    // 子域也认（镜像常挂在 x.cdn.modrinth.com 这类名字下）
    assert!(eligible_host("mediafilez.forgecdn.net"));
    assert!(eligible_host("sub.cdn.modrinth.com"));
}

#[test]
fn test_eligible_host_名单外不认() {
    // 自家 CDN 与公共镜像不做优选：它们本来就直连，钉死 IP 只会更慢
    assert!(!eligible_host("cdn-modrinth.mocdn.net"));
    assert!(!eligible_host("bmclapi2.bangbang93.com"));
    assert!(!eligible_host("mod.mcimirror.top"));
    // 后缀相似但不是子域，不能误判
    assert!(!eligible_host("notcdn.modrinth.com.evil.test"));
    assert!(!eligible_host("cdn.modrinth.com.cn.evil.test"));
}

#[test]
fn test_probe_budget_窗口内累计与耗尽() {
    let start = Instant::now();
    let mut budget = ProbeBudget::new(start);
    assert!(budget.allow(start, 1));
    budget.charge(start, PROBE_BUDGET_BYTES);
    assert!(!budget.allow(start, 1));
    // 窗口滚动后额度恢复
    let later = start + BUDGET_WINDOW;
    assert!(budget.allow(later, PROBE_BUDGET_BYTES));
    assert_eq!(budget.used, 0);
}

#[test]
fn test_probe_budget_未超预算时放行() {
    let start = Instant::now();
    let mut budget = ProbeBudget::new(start);
    budget.charge(start, PROBE_BUDGET_BYTES / 2);
    assert!(budget.allow(start, PROBE_BUDGET_BYTES / 2));
    assert!(!budget.allow(start, PROBE_BUDGET_BYTES / 2 + 1));
}
