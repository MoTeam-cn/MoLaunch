//! log_redact 单元测试

use super::redact_log;

#[test]
fn redacts_toml_quoted_token() {
    assert_eq!(redact_log(r#"token = "secret_value""#), r#"token = "***""#);
}

#[test]
fn redacts_toml_unquoted_token() {
    assert_eq!(redact_log(r#"token=abc123"#), r#"token=***"#);
}

#[test]
fn redacts_json_token() {
    let input = r#"{"token":"secret_value","port":7000}"#;
    let expected = r#"{"token":"***","port":7000}"#;
    assert_eq!(redact_log(input), expected);
}

#[test]
fn redacts_single_quoted_value() {
    assert_eq!(redact_log(r#"token = 'secret'"#), r#"token = '***'"#);
}

#[test]
fn redacts_password_field() {
    assert_eq!(redact_log(r#"password = "mypass""#), r#"password = "***""#);
}

#[test]
fn redacts_secret_field() {
    assert_eq!(redact_log(r#"secret="topsecret""#), r#"secret="***""#);
}

#[test]
fn redacts_api_key_field() {
    assert_eq!(redact_log(r#"api_key: "key123""#), r#"api_key: "***""#);
}

#[test]
fn redacts_auth_token_field() {
    assert_eq!(
        redact_log(r#"auth_token = "abc""#),
        r#"auth_token = "***""#
    );
}

#[test]
fn redacts_case_insensitive() {
    assert_eq!(redact_log(r#"TOKEN = "abc""#), r#"TOKEN = "***""#);
    assert_eq!(redact_log(r#"Password = "abc""#), r#"Password = "***""#);
}

#[test]
fn preserves_log_structure() {
    let line = r#"[2026-07-31 12:00:00.123] [INFO] login with token = "abc" ok"#;
    let result = redact_log(line);
    assert!(result.starts_with("[2026-07-31 12:00:00.123] [INFO] login with token = "));
    assert!(result.contains("\"***\""));
    assert!(result.ends_with(" ok"));
    assert!(!result.contains("abc"));
}

#[test]
fn does_not_redact_non_sensitive_fields() {
    let line = r#"server_addr = "example.com""#;
    assert_eq!(redact_log(line), line);
}

#[test]
fn does_not_redact_prefixed_key() {
    // my_token 中 `_` 是单词字符，`\b` 不在 `_` 与 `t` 之间产生边界，
    // 因此不会误匹配 `my_token` 内部的 `token`。
    let line = r#"my_token = "value""#;
    assert_eq!(redact_log(line), line);
}

#[test]
fn handles_multiple_sensitive_fields_in_one_line() {
    let line = r#"token = "a", password = "b""#;
    let result = redact_log(line);
    assert_eq!(result, r#"token = "***", password = "***""#);
}

#[test]
fn empty_line_unchanged() {
    assert_eq!(redact_log(""), "");
}

#[test]
fn line_without_sensitive_data_unchanged() {
    let line = "[INFO] frpc started on port 7000";
    assert_eq!(redact_log(line), line);
}
