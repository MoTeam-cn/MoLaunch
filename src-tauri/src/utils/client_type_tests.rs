//! client_type 单元测试

use super::*;

#[test]
fn channel_mapping() {
    assert_eq!(channel_code("1.0.0"), 0);
    assert_eq!(channel_code("1.0.0-rc1"), 1);
    assert_eq!(channel_code("1.0.0-rc.1"), 1);
    assert_eq!(channel_code("0.1.0-beta.1"), 2);
    assert_eq!(channel_code("0.1.0-alpha.2"), 3);
    assert_eq!(channel_code("0.1.0-dev"), 3);
    assert_eq!(channel_code("0.2.0-nightly.5"), 4);
    // 未知后缀 → 开发版兜底
    assert_eq!(channel_code("0.1.0-foo"), 3);
}

#[test]
fn main_version_strips_suffix() {
    assert_eq!(main_version("1.0.0"), "1.0.0");
    assert_eq!(main_version("1.0.0-rc1"), "1.0.0");
    assert_eq!(main_version("0.1.0-beta.1"), "0.1.0");
}

#[test]
fn ua_format() {
    let ua = user_agent();
    // 格式：Molaunch/{x.y.z}.{clientType}
    assert!(ua.starts_with("Molaunch/"), "UA: {}", ua);
    let body = &ua["Molaunch/".len()..];
    // 版本.两位编码
    let parts: Vec<&str> = body.split('.').collect();
    assert_eq!(parts.len(), 4, "UA body: {}", body);
    // 最后一位是两位编码
    assert_eq!(parts[3].len(), 2, "UA clientType: {}", parts[3]);
    // 版本部分不含预发布后缀
    assert!(!body.contains('-'), "UA body: {}", body);
}
