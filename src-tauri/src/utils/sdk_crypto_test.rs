//! SDK 加解密封装测试（只覆盖不依赖动态库的部分）

use super::sdk_crypto::{load_error, set_load_error, unavailable_message};

#[test]
fn test_load_error_roundtrip() {
    set_load_error("模拟失败：动态库被杀软拦截".to_string());
    assert_eq!(load_error().as_deref(), Some("模拟失败：动态库被杀软拦截"));
}

#[test]
fn test_unavailable_message_contains_reason_and_path() {
    set_load_error("Failed to load library: os error 193".to_string());
    let message = unavailable_message("加密", "认证数据");
    assert!(message.contains("SDK 未加载"));
    assert!(message.contains("认证数据"));
    assert!(message.contains("Failed to load library"));
    assert!(message.contains("动态库："));
    assert!(message.contains("白名单"));
}
