//! http_log 单元测试

use super::*;

#[test]
fn test_parse_log_line() {
    let line = "[2026-07-29 19:47:32.123] POST /v3/auth/refresh 200 req_id=2026072919478SNCOE6PWP";
    let entry = parse_log_line(line).unwrap();
    assert_eq!(entry.timestamp, "2026-07-29 19:47:32.123");
    assert_eq!(entry.method, "POST");
    assert_eq!(entry.path, "/v3/auth/refresh");
    assert_eq!(entry.status, 200);
    assert_eq!(entry.req_id, "2026072919478SNCOE6PWP");
}

#[test]
fn test_parse_log_line_no_req_id() {
    let line = "[2026-07-29 19:47:32.123] GET /v3/csrf/token 200";
    let entry = parse_log_line(line).unwrap();
    assert_eq!(entry.method, "GET");
    assert_eq!(entry.path, "/v3/csrf/token");
    assert_eq!(entry.status, 200);
    assert_eq!(entry.req_id, "");
}

#[test]
fn test_parse_invalid_line() {
    assert!(parse_log_line("not a log line").is_none());
    assert!(parse_log_line("").is_none());
}

#[test]
fn test_extract_req_id() {
    let body = r#"{"code":1,"data":null,"msg":"ok","req_id":"ABC123"}"#;
    assert_eq!(extract_req_id(body), "ABC123");

    let body_no_id = r#"{"code":1,"data":null,"msg":"ok"}"#;
    assert_eq!(extract_req_id(body_no_id), "");

    assert_eq!(extract_req_id("not json"), "");
}
