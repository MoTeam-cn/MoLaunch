//! 版本声明匹配单测

use super::{compare, matches_game};

#[test]
fn test_compare_按数字段比较() {
    assert_eq!(compare("1.20.1", "1.20.1"), 0);
    assert_eq!(compare("1.20.2", "1.20.1"), 1);
    assert_eq!(compare("1.9", "1.10"), -1, "9 小于 10，不能按字符串比");
}

#[test]
fn test_compare_忽略预发布后缀() {
    assert_eq!(compare("1.20.1-pre1", "1.20.1"), 0);
}

#[test]
fn test_matches_精确相等() {
    assert!(matches_game("1.20.1", "1.20.1"));
    assert!(!matches_game("1.20.1", "1.20.2"));
}

#[test]
fn test_matches_段通配() {
    assert!(
        matches_game("1.20.x", "1.20.1"),
        "原先用字符串相等时这条永远不成立"
    );
    assert!(matches_game("1.20.x", "1.20.4"));
    assert!(!matches_game("1.20.x", "1.21"));
    assert!(matches_game("*", "1.20.1"));
}

#[test]
fn test_matches_比较符区间() {
    assert!(matches_game(">=1.20 <1.21", "1.20.4"));
    assert!(!matches_game(">=1.20 <1.21", "1.21"));
    assert!(!matches_game(">=1.20 <1.21", "1.19.4"));
    assert!(matches_game(">=1.20", "1.20.1"));
}

#[test]
fn test_matches_看不懂的声明不算匹配() {
    assert!(!matches_game("", "1.20.1"), "空声明不能当成全兼容");
    assert!(!matches_game("   ", "1.20.1"));
}
