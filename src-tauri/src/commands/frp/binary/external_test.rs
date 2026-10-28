//! 外部厂商 frpc 下载的安全校验测试

use super::{host_matches, validate_download_url, validate_target_path};

fn domains() -> Vec<String> {
    vec!["example.com".to_string(), "*.cdn.example.com".to_string()]
}

#[test]
fn test_validate_download_url_拒绝非_https() {
    assert!(validate_download_url("http://example.com/frpc.zip", &domains()).is_err());
}

#[test]
fn test_validate_download_url_接受白名单内域名() {
    assert!(validate_download_url("https://example.com/a.zip", &domains()).is_ok());
    assert!(validate_download_url("https://node.cdn.example.com/a.zip", &domains()).is_ok());
}

#[test]
fn test_validate_download_url_拒绝白名单外域名() {
    assert!(validate_download_url("https://evil.com/a.zip", &domains()).is_err());
    // 后缀相同但域名不同的情况不能被通配符放过
    assert!(validate_download_url("https://evilexample.com/a.zip", &domains()).is_err());
}

#[test]
fn test_host_matches_通配符只匹配子域() {
    assert!(host_matches("a.cdn.example.com", "*.cdn.example.com"));
    assert!(!host_matches("cdn.example.com", "*.cdn.example.com"));
    assert!(!host_matches("evilcdn.example.com", "*.cdn.example.com"));
    assert!(!host_matches("example.com.evil.net", "*.example.com"));
}

#[test]
fn test_validate_target_path_接受正常相对路径() {
    assert!(validate_target_path("frpc.exe").is_ok());
    assert!(validate_target_path("bin/frpc").is_ok());
}

#[test]
fn test_validate_target_path_拒绝上级目录() {
    assert!(validate_target_path("../frpc.exe").is_err());
    assert!(validate_target_path("bin/../../frpc.exe").is_err());
}

#[test]
fn test_validate_target_path_拒绝绝对路径与盘符() {
    assert!(validate_target_path("C:/Windows/System32/frpc.exe").is_err());
    assert!(validate_target_path("/etc/frpc").is_err());
    assert!(validate_target_path("C:\\frpc.exe").is_err());
}

#[test]
fn test_validate_target_path_拒绝空路径() {
    assert!(validate_target_path("   ").is_err());
}
