use super::*;

#[test]
fn test_build_v1_toml_basic() {
    let conn = ServerConn {
        server_addr: "hk-6.qwq.fan".to_string(),
        server_port: 17000,
        user: Some("60".to_string()),
        token: Some("va3xljq0469rzujuwzapt1fdmkoiiu32".to_string()),
        use_tls: false,
    };
    let proxies = vec![Proxy {
        name: "my-tunnel".to_string(),
        proxy_type: "tcp".to_string(),
        local_ip: "127.0.0.1".to_string(),
        local_port: 3000,
        remote_port: 30919,
        custom_domains: None,
        bandwidth_limit: Some("4MB".to_string()),
        bandwidth_limit_mode: Some("server".to_string()),
        use_encryption: None,
        use_compression: None,
        protocol_version: None,
    }];
    let toml = build_frpc_toml(&conn, &proxies);
    assert!(toml.contains("serverAddr = 'hk-6.qwq.fan'"));
    assert!(toml.contains("serverPort = 17000"));
    assert!(toml.contains("user = '60'"));
    assert!(toml.contains("[auth]"));
    assert!(toml.contains("token = 'va3xljq0469rzujuwzapt1fdmkoiiu32'"));
    assert!(toml.contains("[[proxies]]"));
    assert!(toml.contains("name = 'my-tunnel'"));
    assert!(toml.contains("type = 'tcp'"));
    assert!(toml.contains("localIP = '127.0.0.1'"));
    assert!(toml.contains("localPort = 3000"));
    assert!(toml.contains("remotePort = 30919"));
    assert!(toml.contains("[proxies.transport]"));
    assert!(toml.contains("bandwidthLimit = '4MB'"));
    assert!(toml.contains("bandwidthLimitMode = 'server'"));
}

#[test]
fn test_build_v1_toml_no_token_no_transport() {
    let conn = ServerConn {
        server_addr: "1.2.3.4".to_string(),
        server_port: 7000,
        user: None,
        token: None,
        use_tls: false,
    };
    let proxies = vec![Proxy {
        name: "p".to_string(),
        proxy_type: "tcp".to_string(),
        local_ip: "127.0.0.1".to_string(),
        local_port: 8080,
        remote_port: 9090,
        custom_domains: None,
        bandwidth_limit: None,
        bandwidth_limit_mode: None,
        use_encryption: None,
        use_compression: None,
        protocol_version: None,
    }];
    let toml = build_frpc_toml(&conn, &proxies);
    assert!(toml.contains("serverAddr = '1.2.3.4'"));
    assert!(!toml.contains("[auth]"));
    assert!(!toml.contains("[proxies.transport]"));
    assert!(!toml.contains("user ="));
}

#[test]
fn test_build_v1_toml_escapes_quote() {
    let conn = ServerConn {
        server_addr: "a'b".to_string(),
        server_port: 1,
        user: None,
        token: Some("t'k".to_string()),
        use_tls: false,
    };
    let toml = build_frpc_toml(&conn, &[]);
    assert!(toml.contains("serverAddr = 'a''b'"));
    assert!(toml.contains("token = 't''k'"));
}

#[test]
fn test_overlay_extra_fields_appends_transport() {
    let raw = "serverAddr = 'hk-6.qwq.fan'\nserverPort = 17000\n";
    let result = overlay_extra_fields(raw, Some("4MB"), Some("server"));
    assert!(result.starts_with("serverAddr = 'hk-6.qwq.fan'"));
    assert!(result.contains("[proxies.transport]"));
    assert!(result.contains("bandwidthLimit = '4MB'"));
    assert!(result.contains("bandwidthLimitMode = 'server'"));
}

#[test]
fn test_overlay_extra_fields_skips_if_exists() {
    let raw = "serverAddr = 'x'\n\n[proxies.transport]\nbandwidthLimit = '8MB'\n";
    let result = overlay_extra_fields(raw, Some("4MB"), Some("server"));
    // 已有 transport 子表，不叠加
    assert!(!result.contains("bandwidthLimit = '4MB'"));
    assert!(result.contains("bandwidthLimit = '8MB'"));
}

#[test]
fn test_overlay_extra_fields_noop_when_empty() {
    let raw = "serverAddr = 'x'\n";
    let result = overlay_extra_fields(raw, None, None);
    assert_eq!(result.trim_end(), raw.trim_end());
}