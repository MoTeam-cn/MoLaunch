//! 快照纯逻辑测试（扫描 / 差异）

use super::snapshot::{diff_files, scan_mods, SnapshotFile};

fn file(name: &str, size: u64, enabled: bool) -> SnapshotFile {
    SnapshotFile {
        name: name.to_string(),
        size,
        enabled,
    }
}

#[test]
fn test_scan_mods_returns_empty_when_dir_missing() {
    let dir = std::env::temp_dir().join("molaunch-snapshot-missing-dir");
    let _ = std::fs::remove_dir_all(&dir);
    let files = scan_mods(&dir).expect("missing dir should be ok");
    assert!(files.is_empty());
}

#[test]
fn test_scan_mods_collects_jar_and_disabled_state() {
    let dir = std::env::temp_dir().join("molaunch-snapshot-scan");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create dir");
    std::fs::write(dir.join("a.jar"), b"12").expect("write a");
    std::fs::write(dir.join("b.jar.disabled"), b"123").expect("write b");
    std::fs::write(dir.join("readme.txt"), b"x").expect("write txt");

    let files = scan_mods(&dir).expect("scan");
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].name, "a.jar");
    assert_eq!(files[0].size, 2);
    assert!(files[0].enabled);
    assert_eq!(files[1].name, "b.jar");
    assert!(!files[1].enabled);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_diff_files_reports_missing_added_and_toggled() {
    let snapshot = vec![
        file("a.jar", 1, true),
        file("b.jar", 1, true),
        file("c.jar", 1, true),
    ];
    let current = vec![
        file("a.jar", 1, true),
        file("b.jar", 1, false),
        file("d.jar", 1, true),
    ];
    let diff = diff_files(&snapshot, &current);
    assert_eq!(diff.missing, vec!["c.jar".to_string()]);
    assert_eq!(diff.added, vec!["d.jar".to_string()]);
    assert_eq!(diff.toggled, vec!["b.jar".to_string()]);
}

#[test]
fn test_diff_files_reports_no_change_when_identical() {
    let snapshot = vec![file("a.jar", 1, true)];
    let current = vec![file("a.jar", 1, true)];
    let diff = diff_files(&snapshot, &current);
    assert!(diff.missing.is_empty());
    assert!(diff.added.is_empty());
    assert!(diff.toggled.is_empty());
}
