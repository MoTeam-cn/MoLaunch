//! 遥测模块测试：签名归一化 / 文本脱敏 / 字段截断 / 请求体字段

use super::signature::{normalize_message, sanitize_text, signature};
use super::types::{truncate_chars, truncate_tail_chars, EventKind, TelemetryEvent};

#[test]
fn test_normalize_message_replaces_numbers() {
    let a = normalize_message("下载失败: 404 在 3 次重试后");
    let b = normalize_message("下载失败: 500 在 9 次重试后");
    assert_eq!(a, b);
    assert!(a.contains("<n>"));
}

#[test]
fn test_normalize_message_replaces_uuid_and_hex() {
    let text = "id=550e8400-e29b-41d4-a716-446655440000 addr=0x7ffb1234";
    let out = normalize_message(text);
    assert!(out.contains("<uuid>"));
    assert!(out.contains("<hex>"));
}

#[test]
fn test_signature_stable_for_same_error_shape() {
    let first = signature("src/a.rs:10", "连接超时 3000ms");
    let second = signature("src/a.rs:10", "连接超时 8000ms");
    assert_eq!(first, second);
    assert_eq!(first.len(), 16);
}

#[test]
fn test_signature_differs_for_different_module() {
    let first = signature("src/a.rs:10", "连接超时");
    let second = signature("src/b.rs:10", "连接超时");
    assert_ne!(first, second);
}

#[test]
fn test_sanitize_text_masks_windows_username() {
    let out = sanitize_text(r"读取失败: C:\Users\XiaoMo\Desktop\a.txt");
    assert!(out.contains("<user>"));
    assert!(!out.contains("XiaoMo"));
}

#[test]
fn test_truncate_chars_keeps_utf8_intact() {
    let text = "中文日志内容测试";
    let out = truncate_chars(text, 3);
    assert_eq!(out, "中文日…");
}

#[test]
fn test_truncate_chars_keeps_short_text() {
    assert_eq!(truncate_chars("短", 10), "短");
}

#[test]
fn test_truncate_tail_chars_keeps_last_chars() {
    let out = truncate_tail_chars("第一行\n第二行\n第三行", 5);
    assert!(out.ends_with("第三行"));
    assert!(out.starts_with('…'));
}

#[test]
fn test_log_tail_keeps_recent_lines() {
    let tail = "old line 1\nold line 2\nNEW ERROR HERE";
    let event = TelemetryEvent::new(EventKind::Error, "src/a.rs:1", "err").with_log_tail(tail);
    assert!(event.log_tail.contains("NEW ERROR HERE"));
}

#[test]
fn test_event_payload_contains_sls_fields() {
    let event =
        TelemetryEvent::new(EventKind::Error, "src/a.rs:10", "加载失败").with_log_tail("line1");
    let payload = event.to_payload("device123");
    assert_eq!(payload["__topic__"], "launcher-error");
    assert_eq!(payload["__source__"], "device123");
    assert_eq!(payload["kind"], "error");
    assert_eq!(payload["device_id"], "device123");
    assert_eq!(payload["log_tail"], "line1");
    assert_eq!(payload["signature"].as_str().map(str::len), Some(16));
    assert!(payload["app_version"]
        .as_str()
        .is_some_and(|v| !v.is_empty()));
}

#[test]
fn test_event_message_is_sanitized_and_truncated() {
    let long = format!("{}", "错".repeat(2000));
    let event = TelemetryEvent::new(EventKind::Error, "src/a.rs:1", &long);
    assert!(event.message.chars().count() <= super::config::MAX_MESSAGE_CHARS + 1);
}
