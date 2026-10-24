//! Java 候选路径优先级判定测试

use super::{has_symlink_component, is_deprioritized_path};
use std::path::Path;

#[test]
fn test_is_deprioritized_path_system32_路径应降权() {
    assert!(is_deprioritized_path(Path::new(
        r"C:\Windows\System32\javaw.exe"
    )));
}

#[test]
fn test_is_deprioritized_path_javapath_target_应降权() {
    assert!(is_deprioritized_path(Path::new(
        r"C:\ProgramData\Oracle\Java\javapath_target_123456\javaw.exe"
    )));
}

#[test]
fn test_is_deprioritized_path_javatmp_应降权() {
    assert!(is_deprioritized_path(Path::new(
        r"C:\Users\XiaoMo\AppData\Local\Temp\javatmp\bin\javaw.exe"
    )));
}

#[test]
fn test_is_deprioritized_path_普通安装路径不降权() {
    assert!(!is_deprioritized_path(Path::new(
        r"C:\Program Files\Java\jdk-17\bin\javaw.exe"
    )));
}

#[test]
fn test_is_deprioritized_path_路径比较不区分大小写() {
    assert!(is_deprioritized_path(Path::new(
        r"C:\WINDOWS\SYSTEM32\javaw.exe"
    )));
}

#[test]
fn test_has_symlink_component_普通路径返回_false() {
    let temp = std::env::temp_dir();
    assert!(!has_symlink_component(&temp));
}

#[test]
fn test_has_symlink_component_不存在的路径返回_false() {
    assert!(!has_symlink_component(Path::new(
        r"C:\__不存在的目录__\bin\javaw.exe"
    )));
}
