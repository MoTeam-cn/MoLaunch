use super::*;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("molaunch-audit-{}", name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("建临时目录");
    dir
}

#[test]
fn test_空报告视为完整() {
    let report = AuditReport::default();
    assert!(report.is_clean());
    assert!(!report.needs_processor_rerun());
    assert_eq!(report.summary(), "产物完整");
}

#[test]
fn test_缺本地生成文件需要重跑processor() {
    let mut report = AuditReport::default();
    report
        .missing_generated
        .push("net.minecraft:client:26.3-1:srg".to_string());
    assert!(!report.is_clean());
    assert!(report.needs_processor_rerun());
    assert!(report.summary().contains("本地生成"));
}

#[test]
fn test_只缺普通库不算需要重跑() {
    let mut report = AuditReport::default();
    report.missing_regular.push("some:lib:1.0".to_string());
    assert!(!report.is_clean());
    assert!(!report.needs_processor_rerun());
}

#[test]
fn test_临吪目录识别() {
    let dir = temp_dir("stray");
    let versions = dir.join("versions");
    for name in [
        "neoforge-26.3.0.75-beta",
        "26.3-NeoForge_26.3.0.75-beta",
        "我的整合",
    ] {
        let v = versions.join(name);
        std::fs::create_dir_all(&v).expect("建版本目录");
        std::fs::write(v.join(format!("{}.json", name)), "{}").expect("写版本 JSON");
    }
    let found = find_stray_loader_dirs(&dir, "neoforge", &["26.3-NeoForge_26.3.0.75-beta"]);
    assert_eq!(found, vec!["neoforge-26.3.0.75-beta".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_不误伤用户自定义名字() {
    let dir = temp_dir("keep");
    let versions = dir.join("versions");
    for name in ["1.20.1-Forge-生存", "forge-test"] {
        let v = versions.join(name);
        std::fs::create_dir_all(&v).expect("建版本目录");
        std::fs::write(v.join(format!("{}.json", name)), "{}").expect("写版本 JSON");
    }
    let found = find_stray_loader_dirs(&dir, "forge", &[]);
    // 只有以 forge- 开头的才算安装器命名
    assert_eq!(found, vec!["forge-test".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}
