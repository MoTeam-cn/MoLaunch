//! easytier join 参数组拼单测（build_join_args 不启动子进程）

use super::easytier::build_join_args;

fn has_pair(args: &[String], key: &str, value: &str) -> bool {
    args.iter()
        .position(|a| a == key)
        .is_some_and(|i| args.get(i + 1).map(|v| v == value).unwrap_or(false))
}

#[test]
fn no_tun_appends_flag() {
    let args = build_join_args("net", "secret", "host", "127.0.0.1:10001", None, true, &[]);
    assert_eq!(args.last().map(|s| s.as_str()), Some("--no-tun"));
}

#[test]
fn tun_mode_omits_flag() {
    let args = build_join_args("net", "secret", "host", "127.0.0.1:10001", None, false, &[]);
    assert!(!args.iter().any(|a| a == "--no-tun"));
}

#[test]
fn host_mode_fixed_ip() {
    let args = build_join_args(
        "net",
        "secret",
        "host",
        "127.0.0.1:10001",
        Some("10.144.144.1"),
        true,
        &[],
    );
    assert!(has_pair(&args, "-i", "10.144.144.1"));
    assert!(!args.iter().any(|a| a == "--dhcp"));
}

#[test]
fn guest_mode_dhcp() {
    let args = build_join_args("net", "secret", "host", "127.0.0.1:10001", None, true, &[]);
    assert!(args.iter().any(|a| a == "--dhcp"));
    assert!(!has_pair(&args, "-i", "10.144.144.1"));
}

#[test]
fn extra_passthrough_before_no_tun() {
    let extra = vec![
        "--peers".to_string(),
        "1.2.3.4:11010".to_string(),
        "--tcp-whitelist".to_string(),
        "13448,25565".to_string(),
        "--udp-whitelist".to_string(),
        "25565".to_string(),
    ];
    let args = build_join_args(
        "net",
        "secret",
        "host",
        "127.0.0.1:10001",
        None,
        true,
        &extra,
    );
    for want in [
        "--peers",
        "1.2.3.4:11010",
        "--tcp-whitelist",
        "13448,25565",
        "--udp-whitelist",
        "25565",
    ] {
        assert!(args.iter().any(|a| a == want), "缺少参数 {want}: {args:?}");
    }
    assert_eq!(args.last().map(|s| s.as_str()), Some("--no-tun"));
}
