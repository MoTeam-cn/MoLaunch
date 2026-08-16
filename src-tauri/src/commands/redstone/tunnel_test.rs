//! hongshi tunnel.ini 状态解析单元测试

use super::parse_tunnel_status;

#[test]
fn parse_open_tunnel() {
    let ts = parse_tunnel_status(
        "[tunnel]\nstatus=open\nserver=hk.hongshi.site\nport=32123\ncreated=2026-08-16 12:00:00",
    );
    assert_eq!(ts.status, "open");
    assert_eq!(ts.server.as_deref(), Some("hk.hongshi.site"));
    assert_eq!(ts.port, Some(32123));
    assert_eq!(ts.created.as_deref(), Some("2026-08-16 12:00:00"));
}

#[test]
fn parse_closed_tunnel_port_none() {
    let ts = parse_tunnel_status(
        "[tunnel]\nstatus=closed\nserver=hk.hongshi.site\nport=-1\ncreated=2026-08-16 12:00:00",
    );
    assert_eq!(ts.status, "closed");
    assert_eq!(ts.server.as_deref(), Some("hk.hongshi.site"));
    assert_eq!(ts.port, None);
    assert_eq!(ts.created.as_deref(), Some("2026-08-16 12:00:00"));
}

#[test]
fn parse_missing_fields() {
    let ts = parse_tunnel_status("[tunnel]\nstatus=open");
    assert_eq!(ts.status, "open");
    assert_eq!(ts.server, None);
    assert_eq!(ts.port, None);
    assert_eq!(ts.created, None);
}

#[test]
fn parse_garbage_content() {
    let ts = parse_tunnel_status("not an ini");
    assert_eq!(ts.status, "unknown");
    assert_eq!(ts.server, None);
    assert_eq!(ts.port, None);
    assert_eq!(ts.created, None);
}

#[test]
fn parse_case_insensitive() {
    let ts = parse_tunnel_status("[Tunnel]\nSTATUS=OPEN\nServer=hk.hongshi.site\nPort=4567");
    assert_eq!(ts.status, "open");
    assert_eq!(ts.server.as_deref(), Some("hk.hongshi.site"));
    assert_eq!(ts.port, Some(4567));
}
