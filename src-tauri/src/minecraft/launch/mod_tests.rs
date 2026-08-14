//! launch 单元测试

use super::*;

#[test]
fn test_sanitize_args() {
    let args = vec![
        "--username".to_string(),
        "player".to_string(),
        "--accessToken".to_string(),
        "eyJhbGciOiJIUzI1NiJ9.secret.token".to_string(),
        "--uuid".to_string(),
        "abc-123".to_string(),
        "--version".to_string(),
        "1.16.5".to_string(),
    ];
    let sanitized = sanitize_args_for_log(&args);
    assert_eq!(sanitized[1], "player");
    assert_eq!(sanitized[3], "***"); // accessToken 值脱敏
    assert_eq!(sanitized[5], "***"); // uuid 值脱敏
    assert_eq!(sanitized[7], "1.16.5"); // 普通参数不脱敏
}

#[test]
fn test_auth_info_debug() {
    let auth = AuthInfo {
        username: "test".to_string(),
        uuid: "uuid".to_string(),
        access_token: "secret_token".to_string(),
        client_token: "client_secret".to_string(),
        login_type: "Microsoft".to_string(),
        server_url: None,
        xuid: String::new(),
    };
    let debug_str = format!("{:?}", auth);
    assert!(debug_str.contains("***"));
    assert!(!debug_str.contains("secret_token"));
    assert!(!debug_str.contains("client_secret"));
}
