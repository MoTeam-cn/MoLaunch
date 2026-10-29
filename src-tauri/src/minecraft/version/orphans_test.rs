//! 孤立目录探测单测

use super::{dir_size, scan_version_orphans, summarize, OrphanDir};

fn temp_root(tag: &str) -> std::path::PathBuf {
    let root =
        std::env::temp_dir().join(format!("molaunch-orphans-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("建临时目录");
    root
}

#[test]
fn test_scan_只报没有同名_json_的目录() {
    let root = temp_root("scan");
    // 正常的版本目录
    std::fs::create_dir_all(root.join("1.20.1")).expect("建目录");
    std::fs::write(root.join("1.20.1").join("1.20.1.json"), b"{}").expect("写 json");
    // 残留目录
    std::fs::create_dir_all(root.join("half-installed")).expect("建目录");
    std::fs::write(root.join("half-installed").join("a.jar"), b"1234").expect("写文件");

    let orphans = scan_version_orphans(&root);
    assert_eq!(orphans.len(), 1, "只该报出那个没有 JSON 的目录");
    assert_eq!(orphans[0].dir_name, "half-installed");
    assert_eq!(orphans[0].size_bytes, 4);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_scan_目录不存在时返回空() {
    let missing = std::env::temp_dir().join("molaunch-orphans-not-there");
    assert!(
        scan_version_orphans(&missing).is_empty(),
        "读不到就当没有，不要 panic"
    );
}

#[test]
fn test_dir_size_递归求和() {
    let root = temp_root("size");
    std::fs::create_dir_all(root.join("nested")).expect("建目录");
    std::fs::write(root.join("top.bin"), b"12345").expect("写文件");
    std::fs::write(root.join("nested").join("inner.bin"), b"123").expect("写文件");
    assert_eq!(dir_size(&root), 8);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_summarize_空清单返回_none() {
    assert!(summarize(&[]).is_none(), "没东西可说就别说话");
    let one = OrphanDir {
        dir_name: "x".to_string(),
        size_bytes: 3 * 1024 * 1024,
        reason: "test".to_string(),
    };
    let text = summarize(&[one]).expect("应有一句话");
    assert!(
        text.contains("1 个") && text.contains("3 MB"),
        "实际: {}",
        text
    );
}
