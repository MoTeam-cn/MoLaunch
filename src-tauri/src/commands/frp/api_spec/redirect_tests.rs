//! redirect 单元测试

use super::*;

#[test]
fn test_is_scheme_downgrade_https_to_http_is_downgrade() {
    assert!(is_scheme_downgrade(
        "https://api.example.com/v1",
        "http://api.example.com/v1"
    ));
}

#[test]
fn test_is_scheme_downgrade_https_to_https_is_not_downgrade() {
    assert!(!is_scheme_downgrade(
        "https://api.example.com/v1",
        "https://api.example.com/v2"
    ));
}

#[test]
fn test_is_scheme_downgrade_http_base_is_not_flagged() {
    // 基准本身非 https 时不算"降级"（正常路径下 baseUrl 已被 registry 强制 https）
    assert!(!is_scheme_downgrade(
        "http://api.example.com/v1",
        "http://api.example.com/v2"
    ));
}

#[test]
fn test_resolve_url_absolute_location_returned_as_is() {
    assert_eq!(
        resolve_url("https://api.example.com/v1", "http://api.example.com/v2").unwrap(),
        "http://api.example.com/v2"
    );
}

#[test]
fn test_resolve_url_relative_location_keeps_base_scheme() {
    assert_eq!(
        resolve_url("https://api.example.com/v1/a", "/v1/b").unwrap(),
        "https://api.example.com/v1/b"
    );
    // 相对路径按主机根解析（非 base 路径），保持 https
    assert_eq!(
        resolve_url("https://api.example.com/v1/a", "b").unwrap(),
        "https://api.example.com/b"
    );
}

#[test]
fn test_extract_host_strips_scheme_and_port() {
    assert_eq!(
        extract_host("https://API.Example.com:8443/x").as_deref(),
        Some("api.example.com")
    );
    assert_eq!(
        extract_host("http://127.0.0.1:8080/x").as_deref(),
        Some("127.0.0.1")
    );
    assert_eq!(extract_host("ftp://x/y"), None);
}
