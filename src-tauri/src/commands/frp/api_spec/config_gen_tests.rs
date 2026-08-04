//! config_gen 单元测试

use super::*;

fn test_tunnel() -> TunnelInfo {
    TunnelInfo {
        id: "t1".to_string(),
        name: "test".to_string(),
        tunnel_type: "tcp".to_string(),
        status: "running".to_string(),
        server_host: "example.com".to_string(),
        server_port: "7000".to_string(),
        token: "secret".to_string(),
        local_host: "127.0.0.1".to_string(),
        local_port: "25565".to_string(),
        remote_port: "25565".to_string(),
        ..Default::default()
    }
}

#[test]
fn test_fields_mode() {
    let tunnel = test_tunnel();
    let account = AccountInfo::default();
    let result = generate("fields", "ini", &[], &tunnel, &account, None, None).unwrap();
    let content = result.content.unwrap();
    assert!(content.contains("server_addr = example.com"));
    assert!(content.contains("server_port = 7000"));
    assert!(content.contains("[test]"));
    assert!(content.contains("type = tcp"));
}

#[test]
fn test_args_mode() {
    let tunnel = test_tunnel();
    let account = AccountInfo::default();
    let template = vec![
        "-u".to_string(),
        "{token}".to_string(),
        "-p".to_string(),
        "{ids}".to_string(),
    ];
    let result = generate("args", "ini", &template, &tunnel, &account, None, None).unwrap();
    assert_eq!(result.args, vec!["-u", "secret", "-p", "t1"]);
}

#[test]
fn test_url_mode() {
    let tunnel = test_tunnel();
    let account = AccountInfo::default();
    let raw = "[common]\nserver_addr = x\n";
    let result = generate("url", "ini", &[], &tunnel, &account, Some(raw), None).unwrap();
    assert_eq!(result.content.unwrap(), raw);
}

#[test]
fn test_url_mode_base64() {
    let tunnel = test_tunnel();
    let account = AccountInfo::default();
    // Base64 编码的 "[common]\nserver_addr = x\n"
    let raw = "W2NvbW1vbl0Kc2VydmVyX2FkZHIgPSB4Cg==";
    let result = generate(
        "url",
        "ini",
        &[],
        &tunnel,
        &account,
        Some(raw),
        Some("base64"),
    )
    .unwrap();
    let content = result.content.unwrap();
    assert!(content.contains("server_addr = x"));
    assert!(content.contains("[common]"));
}
