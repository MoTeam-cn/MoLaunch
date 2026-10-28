//! FileChecker 测试：大小校验、哈希校验，以及「认不出的哈希刻意跳过」这条既定取舍

use super::FileChecker;
use std::path::PathBuf;

fn temp_file(name: &str, content: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("molaunch-filechecker-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    let path = dir.join(name);
    std::fs::write(&path, content).expect("写入临时文件");
    path
}

#[test]
fn test_check_大小不符则失败() {
    let path = temp_file("size.bin", b"0123456789");
    let checker = FileChecker::new().with_actual_size(20);
    assert!(!checker.is_valid(path.to_string_lossy().as_ref()));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_check_哈希不符则失败() {
    let path = temp_file("hash.bin", b"abcdef");
    // 40 位是 sha1，内容和它对不上
    let checker = FileChecker::new()
        .with_actual_size(6)
        .with_hash(Some("0".repeat(40)));
    assert!(!checker.is_valid(path.to_string_lossy().as_ref()));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_check_哈希长度认不出时只校验大小() {
    let path = temp_file("unknown-hash.bin", b"abcdef");
    // 元数据里的哈希长度既不是 32/40/64/128：刻意跳过哈希、只看大小，
    // 否则哈希写坏的老版本会永远重下（这是一条有意的取舍，见 file_checker.rs 注释）
    let checker = FileChecker::new()
        .with_actual_size(6)
        .with_hash(Some("not-a-real-hash".to_string()));
    assert!(
        checker.is_valid(path.to_string_lossy().as_ref()),
        "认不出长度时应当放行，只保留大小校验"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_check_无大小无哈希时不放行() {
    let path = temp_file("no-meta.bin", b"abcdef");
    let checker = FileChecker::new().with_actual_size(0);
    assert!(!checker.is_valid(path.to_string_lossy().as_ref()));
    let _ = std::fs::remove_file(&path);
}
