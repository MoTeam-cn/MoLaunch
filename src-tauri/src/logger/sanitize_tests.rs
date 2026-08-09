//! sanitize 模块单元测试

use super::*;

#[test]
fn test_sanitize_jwt() {
    let input = "Launching with token eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let result = sanitize_sensitive_info(input);
    assert!(!result.contains("eyJhbGciOiJIUzI1NiJ9"));
    assert!(result.contains("***"));
}

#[test]
fn test_sanitize_json_token() {
    let input = r#"Auth response: {"access_token":"eyJsecret12345678","username":"player"}"#;
    let result = sanitize_sensitive_info(input);
    assert!(result.contains(r#""access_token":"***""#));
    assert!(!result.contains("eyJsecret12345678"));
    // username 不应被脱敏
    assert!(result.contains("player"));
}

#[test]
fn test_sanitize_preserves_short_strings() {
    let input = "Game version: 1.16.5, Java path: C:/java/javaw.exe";
    let result = sanitize_sensitive_info(input);
    assert_eq!(input, result);
}

#[test]
fn test_sanitize_preserves_urls() {
    // URL 路径含 `/`，不应被误判为 token
    let input = "https://cdn-modrinth.mocdn.net/data/l9m9tuPN/versions/M8j2mfGj/physics-mod-3.0.14-mc-1.20.1-forge.jar";
    let result = sanitize_sensitive_info(input);
    assert_eq!(input, result, "URL should not be sanitized");
}

#[test]
fn test_sanitize_preserves_texture_url_with_hex_hash() {
    // Minecraft 材质 URL 末尾是 64 字符 hex hash，不应被脱敏
    let input = "https://textures.minecraft.net/texture/a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
    let result = sanitize_sensitive_info(input);
    assert_eq!(input, result, "texture URL hash should not be sanitized");
}

#[test]
fn test_sanitize_password_field() {
    let input = r#"Login request: {"password":"hunter2secret","user":"alice"}"#;
    let result = sanitize_sensitive_info(input);
    assert!(result.contains(r#""password":"***""#));
    assert!(!result.contains("hunter2secret"));
    // 非敏感字段不应被脱敏
    assert!(result.contains("alice"));
}

#[test]
fn test_sanitize_auth_bearer_header() {
    let input = "request headers: Authorization: Bearer ghp_abcdefghij";
    let result = sanitize_sensitive_info(input);
    assert!(result.contains("Authorization: Bearer ***"));
    assert!(!result.contains("ghp_abcdefghij"));
}

#[test]
fn test_sanitize_url_query_token() {
    let input = "http://example.com/api?token=abc123&x=1";
    let result = sanitize_sensitive_info(input);
    assert!(result.contains("?token=***"));
    assert!(!result.contains("abc123"));
    // 非敏感参数应保留
    assert!(result.contains("&x=1"));
}

#[test]
fn test_sanitize_api_key_field() {
    let input = r#"Config: {"api_key":"sk-abcdefgh","name":"server"}"#;
    let result = sanitize_sensitive_info(input);
    assert!(result.contains(r#""api_key":"***""#));
    assert!(!result.contains("sk-abcdefgh"));
}
