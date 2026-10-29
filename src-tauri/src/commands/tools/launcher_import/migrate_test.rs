//! 导入复制单测：普通目录复制、重解析点不跟随、目录成环不死循环

use super::copy_tree;
use std::collections::HashSet;
use std::path::PathBuf;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("molaunch-import-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    dir
}

#[test]
fn test_copy_tree_复制普通目录并跳过元数据文件() {
    let root = temp_root("plain");
    let src = root.join("src");
    std::fs::create_dir_all(src.join("mods")).expect("建源目录");
    std::fs::write(src.join("mods").join("a.jar"), b"jar").expect("写模组文件");
    std::fs::write(src.join("instance.cfg"), b"meta").expect("写启动器元数据");
    let dst = root.join("dst");
    std::fs::create_dir_all(&dst).expect("建目标目录");

    let mut visited = HashSet::new();
    copy_tree(&src, &dst, "src", &mut visited).expect("复制应成功");

    assert_eq!(
        std::fs::read(dst.join("mods").join("a.jar")).expect("读回"),
        b"jar"
    );
    assert!(
        !dst.join("instance.cfg").exists(),
        "启动器元数据不应被复制过去"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn test_copy_tree_重解析点不跟随而是重建链接() {
    let root = temp_root("link");
    let shared = root.join("shared");
    std::fs::create_dir_all(shared.join("inner")).expect("建共享目录");
    std::fs::write(shared.join("inner").join("lib.jar"), vec![7_u8; 4096]).expect("写共享文件");
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("建源目录");
    std::os::unix::fs::symlink(&shared, src.join("libraries")).expect("建源链接");
    let dst = root.join("dst");
    std::fs::create_dir_all(&dst).expect("建目标目录");

    let mut visited = HashSet::new();
    copy_tree(&src, &dst, "src", &mut visited).expect("复制应成功");

    let linked = dst.join("libraries");
    let meta = std::fs::symlink_metadata(&linked).expect("目标应存在");
    assert!(
        meta.file_type().is_symlink(),
        "应当重建链接，而不是把链接指向的内容整份复制过来"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn test_copy_tree_目录成环时不会无限递归() {
    let root = temp_root("cycle");
    let src = root.join("src");
    std::fs::create_dir_all(src.join("mods")).expect("建源目录");
    // mods/loop -> src 自身，形成环
    std::os::unix::fs::symlink(&src, src.join("mods").join("loop")).expect("建环");
    let dst = root.join("dst");
    std::fs::create_dir_all(&dst).expect("建目标目录");

    let mut visited = HashSet::new();
    copy_tree(&src, &dst, "src", &mut visited).expect("遇到环也应正常返回");
    let _ = std::fs::remove_dir_all(&root);
}
