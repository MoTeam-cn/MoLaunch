//! .part 原子替换单测

use super::{backup_path, drop_backup, promote_part, restore_backup, stash_existing};

fn temp_path(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("molaunch-single-{}-{}", tag, std::process::id()))
}

#[test]
fn test_promote_part_替换后内容完整且临时文件消失() {
    let part = temp_path("part.part");
    let target = temp_path("part");
    let _ = std::fs::remove_file(&part);
    let _ = std::fs::remove_file(&target);
    std::fs::write(&part, b"finished").expect("写临时文件");

    promote_part(
        part.to_string_lossy().as_ref(),
        target.to_string_lossy().as_ref(),
    )
    .expect("替换应成功");

    assert_eq!(std::fs::read(&target).expect("读回"), b"finished");
    assert!(!part.exists(), "临时文件应已被移走");
    let _ = std::fs::remove_file(&target);
}

#[test]
fn test_promote_part_覆盖已存在的目标文件() {
    let part = temp_path("over.part");
    let target = temp_path("over");
    std::fs::write(&part, b"new").expect("写临时文件");
    std::fs::write(&target, b"old-content").expect("写旧文件");

    promote_part(
        part.to_string_lossy().as_ref(),
        target.to_string_lossy().as_ref(),
    )
    .expect("替换应成功");

    assert_eq!(std::fs::read(&target).expect("读回"), b"new");
    let _ = std::fs::remove_file(&target);
}

#[test]
fn test_promote_part_临时文件不存在时报错() {
    let missing = temp_path("missing.part");
    let _ = std::fs::remove_file(&missing);
    let result = promote_part(
        missing.to_string_lossy().as_ref(),
        temp_path("missing").to_string_lossy().as_ref(),
    );
    assert!(result.is_err(), "没下完就谈不上替换，必须报错");
}

#[test]
fn test_backup_path_加固定后缀() {
    assert_eq!(backup_path("a/b.jar"), "a/b.jar.molaunch-bak");
}

#[test]
fn test_stash_existing_保留旧内容且可恢复() {
    let target = temp_path("stash");
    std::fs::write(&target, b"old-version").expect("写旧文件");
    let stash = stash_existing(target.to_string_lossy().as_ref()).expect("应生成备份");
    assert!(
        std::path::Path::new(&stash).exists(),
        "备份文件必须真的存在"
    );
    std::fs::write(&target, b"new-version").expect("模拟替换后的新文件");
    assert!(restore_backup(
        Some(stash),
        target.to_string_lossy().as_ref()
    ));
    assert_eq!(
        std::fs::read(&target).expect("读回"),
        b"old-version",
        "恢复的必须是旧内容"
    );
    let _ = std::fs::remove_file(&target);
}

#[test]
fn test_stash_existing_目标不存在时返回_none() {
    let missing = temp_path("no-such-file");
    let _ = std::fs::remove_file(&missing);
    assert!(stash_existing(missing.to_string_lossy().as_ref()).is_none());
}

#[test]
fn test_drop_backup_删掉备份文件() {
    let target = temp_path("drop");
    std::fs::write(&target, b"x").expect("写文件");
    let stash = stash_existing(target.to_string_lossy().as_ref()).expect("应生成备份");
    drop_backup(Some(stash.clone()));
    assert!(!std::path::Path::new(&stash).exists(), "提交后不该留备份");
    let _ = std::fs::remove_file(&target);
}
