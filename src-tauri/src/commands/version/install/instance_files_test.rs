//! 文件事实表单测：差异判定与快照

use super::{diff, sanitize, snapshot, FileFact};

fn fact(path: &str, size: u64, mtime_ms: u64) -> FileFact {
    FileFact {
        path: path.to_string(),
        size,
        mtime_ms,
    }
}

#[test]
fn test_diff_识别新增删除与改写() {
    let prev = vec![fact("a.jar", 10, 1), fact("b.json", 20, 1)];
    let now = vec![
        fact("a.jar", 10, 1),
        fact("b.json", 25, 2),
        fact("mods/c.jar", 5, 3),
    ];
    let d = diff(&prev, &now);
    assert_eq!(d.added, vec!["mods/c.jar".to_string()]);
    assert!(d.removed.is_empty());
    assert_eq!(d.changed, vec!["b.json".to_string()], "大小变了就算改写");
}

#[test]
fn test_diff_完全相同则无差异() {
    let prev = vec![fact("a.jar", 10, 1)];
    assert_eq!(diff(&prev, &prev.clone()), super::FileDiff::default());
}

#[test]
fn test_snapshot_递归并统一分隔符() {
    let root = std::env::temp_dir().join(format!("molaunch-facts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("mods")).expect("建目录");
    std::fs::write(root.join("client.json"), b"{}").expect("写文件");
    std::fs::write(root.join("mods").join("x.jar"), b"12345").expect("写文件");

    let facts = snapshot(&root);
    let paths: Vec<&str> = facts.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"client.json"), "顶层文件要记到");
    assert!(
        paths
            .iter()
            .any(|p| p.ends_with("x.jar") && p.contains('/')),
        "子目录文件要用 / 分隔的相对路径"
    );
    let jar = facts
        .iter()
        .find(|f| f.path.ends_with("x.jar"))
        .expect("应有 x.jar");
    assert_eq!(jar.size, 5);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_sanitize_替换非法字符() {
    assert_eq!(sanitize("1.20.1/forge"), "1.20.1_forge");
    assert_eq!(sanitize("我的版本"), "我的版本");
}
