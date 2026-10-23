//! AI 服务错误响应解析测试

use super::{extract_error_message, http_error_message};

#[test]
fn test_extract_error_message_openai_风格_message_字段() {
    let body = r#"{"error":{"message":"Invalid API key","type":"invalid_request_error"}}"#;
    assert_eq!(
        extract_error_message(body).as_deref(),
        Some("Invalid API key")
    );
}

#[test]
fn test_extract_error_message_error_为字符串时直接取用() {
    let body = r#"{"error":"model not found"}"#;
    assert_eq!(
        extract_error_message(body).as_deref(),
        Some("model not found")
    );
}

#[test]
fn test_extract_error_message_error_为数组时逐条拼接() {
    let body = r#"{"error":[{"message":"字段缺失"},{"message":"字段非法"}]}"#;
    assert_eq!(
        extract_error_message(body).as_deref(),
        Some("字段缺失；字段非法")
    );
}

#[test]
fn test_extract_error_message_errors_字符串数组() {
    let body = r#"{"errors":["超时","重试"]}"#;
    assert_eq!(extract_error_message(body).as_deref(), Some("超时；重试"));
}

#[test]
fn test_extract_error_message_非_json_回退原文并压缩空白() {
    let body = "  502   Bad Gateway\n\n";
    assert_eq!(
        extract_error_message(body).as_deref(),
        Some("502 Bad Gateway")
    );
}

#[test]
fn test_extract_error_message_空响应返回_none() {
    assert!(extract_error_message("   ").is_none());
}

#[test]
fn test_extract_error_message_超长文本截断() {
    let long = "a".repeat(500);
    let body = format!(r#"{{"error":{{"message":"{}"}}}}"#, long);
    let msg = extract_error_message(&body).expect("应能提取");
    assert_eq!(msg.chars().count(), 301);
    assert!(msg.ends_with('…'));
}

#[test]
fn test_http_error_message_拼接状态码与原因() {
    let body = r#"{"error":{"message":"quota exceeded"}}"#;
    assert_eq!(
        http_error_message(429, body),
        "AI 服务返回 HTTP 429: quota exceeded"
    );
}

#[test]
fn test_http_error_message_无详情时给兜底文案() {
    assert_eq!(
        http_error_message(500, ""),
        "AI 服务返回 HTTP 500（服务端未返回错误详情）"
    );
}
