//! Java 验证单测：跑不起来的文件必须返回 Err，而不是只打一行日志

use super::verify_downloaded_java;
use std::path::Path;

#[test]
fn test_verify_downloaded_java_不存在的文件返回错误() {
    let result = verify_downloaded_java(Path::new("/definitely/not/a/java/binary"));
    assert!(
        result.is_err(),
        "路径不存在时必须报错，否则调用方会把坏路径写进配置"
    );
    let message = result.unwrap_err();
    assert!(!message.is_empty(), "错误信息不能是空的");
}
