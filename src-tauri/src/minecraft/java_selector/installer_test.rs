//! 安装器 Java 要求单测

use super::required_java_major;

#[test]
fn test_required_java_major_新版本号体系要_21() {
    assert_eq!(required_java_major("26.3"), 21, "26.x 早已是 Java 21 起步");
    assert_eq!(required_java_major("30.1"), 21);
}

#[test]
fn test_required_java_major_121_要_21() {
    assert_eq!(required_java_major("1.21.1"), 21);
    assert_eq!(required_java_major("1.21"), 21);
}

#[test]
fn test_required_java_major_1205_起要_21() {
    assert_eq!(required_java_major("1.20.6"), 21);
    assert_eq!(required_java_major("1.20.5"), 21);
    assert_eq!(
        required_java_major("1.20.4"),
        17,
        "1.20.4 及以前仍是 Java 17"
    );
}

#[test]
fn test_required_java_major_118_到_1204_要_17() {
    assert_eq!(required_java_major("1.18.2"), 17);
    assert_eq!(required_java_major("1.19.4"), 17);
}

#[test]
fn test_required_java_major_老版本只要_8() {
    assert_eq!(required_java_major("1.16.5"), 8);
    assert_eq!(required_java_major("1.12.2"), 8);
    assert_eq!(required_java_major("1.7.10"), 8);
}

#[test]
fn test_required_java_major_解析不出时按旧版处理() {
    assert_eq!(required_java_major(""), 8);
    assert_eq!(required_java_major("snapshot-abc"), 8);
}
