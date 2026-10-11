//! frpc 日志解析单元测试

use super::extract_log_timestamp;

/// 非 ASCII 行不得 panic
///
/// 回归：曾用字节切片 `&trimmed[..23]`，非 ASCII 行会踩 char boundary 直接 panic，
/// 而本函数对日志的每一行调用。
#[test]
fn test_extract_log_timestamp_非时间戳行不崩溃() {
    for line in [
        "中文中文中文中文中文中文中文",
        "日志：这是一条足够长的中文行用来触发长度判断分支",
        "éééééééééééééééééééééééé",
        "emoji 混排 🎮🎮🎮🎮🎮🎮🎮🎮🎮🎮🎮🎮",
    ] {
        let parsed = extract_log_timestamp(line);
        assert!(parsed.is_empty(), "非时间戳行应返回空串: {:?}", line);
    }
}

/// 标准 frpc 时间戳行仍能正确解析（修复不得改变正常行为）
#[test]
fn test_extract_log_timestamp_解析标准时间戳() {
    assert_eq!(
        extract_log_timestamp("2026-10-11 18:54:22.123 [INFO] hello"),
        "2026-10-11 18:54:22.123"
    );
    assert_eq!(
        extract_log_timestamp("[2026-10-11 18:54:22.123] [INFO] hello"),
        "2026-10-11 18:54:22.123"
    );
}
