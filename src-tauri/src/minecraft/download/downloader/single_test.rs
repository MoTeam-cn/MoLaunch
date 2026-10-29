//! .part 原子替换单测

use super::promote_part;

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
