//! SDK 加解密封装测试（只覆盖不依赖动态库的部分）

use super::sdk_crypto::{load_error, set_load_error, unavailable_message};

/// 全局错误槽是共享状态，两个断言合并为一个用例，避免并行测试互相覆盖
#[test]
fn test_load_error_and_unavailable_message() {
    set_load_error("模拟失败：动态库被杀软拦截".to_string());
    assert_eq!(load_error().as_deref(), Some("模拟失败：动态库被杀软拦截"));

    let message = unavailable_message("加密", "认证数据");
    assert!(message.contains("SDK 未加载"));
    assert!(message.contains("认证数据"));
    assert!(message.contains("模拟失败：动态库被杀软拦截"));
    assert!(message.contains("动态库："));
    assert!(message.contains("白名单"));
}
