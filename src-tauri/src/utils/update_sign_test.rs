//! update_sign 单元测试

use super::{build_ping_string_to_sign, sign_hex, SIGN_LX_HMAC_SHA256};

#[test]
fn test_build_ping_string_to_sign_与服务端逐字一致() {
    // 期望串与 api-server 的 client_sign_test.rs 同名用例**逐字相同**：
    // 两端任一侧改动协议都会让对应测试失败，从而挡住协议漂移。
    let s = build_ping_string_to_sign(
        "11111111-2222-3333-4444-555555555555",
        "mcsdk-windows-0.3.8-rc1-molaunch",
        "1700000000",
        "1",
    );
    assert_eq!(
        s,
        "POST\n/v1/launch/ping\n11111111-2222-3333-4444-555555555555\nmcsdk-windows-0.3.8-rc1-molaunch\n1700000000\n1"
    );
}

#[test]
fn test_sign_hex_长度为64位小写hex() {
    let sig = sign_hex(&"01".repeat(32), "hello").expect("合法密钥应能签名");
    assert_eq!(sig.len(), 64);
    assert!(sig
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

#[test]
fn test_sign_hex_相同输入结果稳定() {
    let key = "ab".repeat(32);
    assert_eq!(sign_hex(&key, "x"), sign_hex(&key, "x"));
}

#[test]
fn test_sign_hex_不同密钥结果不同() {
    assert_ne!(
        sign_hex(&"01".repeat(32), "x"),
        sign_hex(&"02".repeat(32), "x")
    );
}

#[test]
fn test_sign_hex_非法密钥返回_none() {
    assert!(sign_hex("zz", "x").is_none());
    assert!(sign_hex("abc", "x").is_none());
}

#[test]
fn test_sign_lx_常量() {
    assert_eq!(SIGN_LX_HMAC_SHA256, "1");
}
