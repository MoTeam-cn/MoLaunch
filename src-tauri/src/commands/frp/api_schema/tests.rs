//! api_schema 单元测试

use super::helpers::{
    build_url, compute_timeout, escape_toml_string, extract_host, resolve_url,
};
use super::mapping::{get_json_path, map_response};
use super::ConfigPayload;
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_get_json_path() {
    let json: serde_json::Value = serde_json::json!({
        "data": {
            "server_addr": "1.2.3.4",
            "server_port": 7000,
            "nested": { "deep": "value" }
        }
    });
    assert_eq!(
        get_json_path(&json, "data.server_addr"),
        Some(serde_json::json!("1.2.3.4"))
    );
    assert_eq!(
        get_json_path(&json, "data.server_port"),
        Some(serde_json::json!(7000))
    );
    assert_eq!(
        get_json_path(&json, "data.nested.deep"),
        Some(serde_json::json!("value"))
    );
    assert_eq!(get_json_path(&json, "data.missing"), None);
    assert_eq!(get_json_path(&json, "nonexistent.path"), None);
}

#[test]
fn test_build_url() {
    assert_eq!(
        build_url("https://api.x.com", "/v1/config").unwrap(),
        "https://api.x.com/v1/config"
    );
    assert_eq!(
        build_url("https://api.x.com/", "/v1/config").unwrap(),
        "https://api.x.com/v1/config"
    );
    assert_eq!(
        build_url("https://api.x.com", "v1/config").unwrap(),
        "https://api.x.com/v1/config"
    );
    assert_eq!(
        build_url("https://api.x.com", "").unwrap(),
        "https://api.x.com"
    );
    assert_eq!(
        build_url("https://api.x.com", "https://other.com/api").unwrap(),
        "https://other.com/api"
    );
}

#[test]
fn test_compute_timeout() {
    assert_eq!(compute_timeout(None), Duration::from_millis(10_000));
    assert_eq!(compute_timeout(Some(5_000)), Duration::from_millis(5_000));
    assert_eq!(compute_timeout(Some(60_000)), Duration::from_millis(30_000));
    assert_eq!(compute_timeout(Some(100)), Duration::from_millis(1_000));
}

#[test]
fn test_extract_host() {
    assert_eq!(extract_host("https://api.example.com/path"), Some("api.example.com".to_string()));
    assert_eq!(extract_host("https://api.example.com:8080/path"), Some("api.example.com".to_string()));
    assert_eq!(extract_host("http://localhost:3000"), Some("localhost".to_string()));
    assert_eq!(extract_host("not-a-url"), None);
}

#[test]
fn test_resolve_url() {
    assert_eq!(
        resolve_url("https://api.x.com/v1/config", "https://other.com/api").unwrap(),
        "https://other.com/api"
    );
    assert_eq!(
        resolve_url("https://api.x.com/v1/config", "/v2/config").unwrap(),
        "https://api.x.com/v2/config"
    );
    assert_eq!(
        resolve_url("https://api.x.com/v1/config", "config2").unwrap(),
        "https://api.x.com/config2"
    );
}

#[test]
fn test_map_response_standard_fields() {
    let mut mapping = HashMap::new();
    mapping.insert("data.host".to_string(), "serverAddr".to_string());
    mapping.insert("data.port".to_string(), "serverPort".to_string());
    mapping.insert("data.key".to_string(), "token".to_string());

    let response = serde_json::json!({
        "data": { "host": "frps.example.com", "port": 7000, "key": "secret" }
    });

    let payload = map_response(&response, &mapping).unwrap();
    assert_eq!(payload.server_addr, "frps.example.com");
    assert_eq!(payload.server_port, 7000);
    assert_eq!(payload.token, Some("secret".to_string()));
}

#[test]
fn test_map_response_custom_variables() {
    let mut mapping = HashMap::new();
    mapping.insert("data.host".to_string(), "serverAddr".to_string());
    mapping.insert("data.port".to_string(), "serverPort".to_string());
    mapping.insert("data.extra".to_string(), "customVar".to_string());

    let response = serde_json::json!({
        "data": { "host": "x.com", "port": 7000, "extra": "hello" }
    });

    let payload = map_response(&response, &mapping).unwrap();
    assert_eq!(
        payload.custom_variables.and_then(|m| m.get("customVar").cloned()),
        Some("hello".to_string())
    );
}

#[test]
fn test_map_response_missing_required() {
    let mut mapping = HashMap::new();
    mapping.insert("data.host".to_string(), "serverAddr".to_string());
    // 缺少 serverPort

    let response = serde_json::json!({ "data": { "host": "x.com" } });
    let result = map_response(&response, &mapping);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("服务器端口"));
}

#[test]
fn test_render_config_template() {
    let payload = ConfigPayload {
        server_addr: "frps.example.com".to_string(),
        server_port: 7000,
        token: Some("secret\"key".to_string()),
        assigned_remote_port: Some(30001),
        assigned_subdomain: Some("my-tunnel".to_string()),
        custom_variables: None,
    };

    let template = r#"serverAddr = "{server_addr}"
serverPort = {server_port}
auth.token = "{token}"
remotePort = {assigned_remote_port}
subdomain = "{assigned_subdomain}""#;

    // 写入临时文件测试
    let result = template
        .replace("{server_addr}", &escape_toml_string(&payload.server_addr))
        .replace("{server_port}", &payload.server_port.to_string())
        .replace("{token}", &escape_toml_string(payload.token.as_ref().unwrap()))
        .replace("{assigned_remote_port}", &payload.assigned_remote_port.unwrap().to_string())
        .replace("{assigned_subdomain}", &escape_toml_string(payload.assigned_subdomain.as_ref().unwrap()));

    assert!(result.contains("serverAddr = \"frps.example.com\""));
    assert!(result.contains("serverPort = 7000"));
    assert!(result.contains("auth.token = \"secret\\\"key\""));
    assert!(result.contains("remotePort = 30001"));
    assert!(result.contains("subdomain = \"my-tunnel\""));
}
