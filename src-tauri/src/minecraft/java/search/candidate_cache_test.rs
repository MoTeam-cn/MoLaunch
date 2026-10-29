//! 候选缓存纯逻辑单测

use std::path::PathBuf;

use super::{candidates_alive, is_fresh, key_of};

#[test]
fn test_key_of_与顺序无关() {
    let a = vec![PathBuf::from("C:/one"), PathBuf::from("D:/two")];
    let b = vec![PathBuf::from("D:/two"), PathBuf::from("C:/one")];
    assert_eq!(key_of(&a), key_of(&b));
}

#[test]
fn test_key_of_忽略重复与大小写() {
    let a = vec![PathBuf::from("C:/One"), PathBuf::from("c:/one")];
    assert_eq!(key_of(&a), "c:/one");
}

#[test]
fn test_is_fresh_过期判定() {
    let now = 10_000_000_u64;
    assert!(is_fresh(now, now));
    assert!(is_fresh(now - 1000, now));
    assert!(!is_fresh(now - 30 * 60 * 1000 - 1, now));
}

#[test]
fn test_candidates_alive_有死路径即失效() {
    let dir = std::env::temp_dir();
    let alive = dir.join(format!("molaunch-java-alive-{}", std::process::id()));
    std::fs::write(&alive, b"x").expect("写临时文件");
    assert!(candidates_alive(&[alive.to_string_lossy().to_string()]));
    let missing = dir.join("molaunch-java-not-there-xyz");
    assert!(!candidates_alive(&[
        alive.to_string_lossy().to_string(),
        missing.to_string_lossy().to_string()
    ]));
    let _ = std::fs::remove_file(&alive);
}
