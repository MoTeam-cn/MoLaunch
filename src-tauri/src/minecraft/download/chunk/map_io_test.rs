//! 数据刷盘单测：刷盘失败必须报错，不能静默成功

use super::flush_data_file;

fn temp_file(tag: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("molaunch-mapio-{}-{}", tag, std::process::id()));
    std::fs::write(&path, b"payload").expect("写临时文件");
    path
}

#[test]
fn test_flush_data_file_写入后落盘成功且内容不变() {
    let path = temp_file("ok");
    flush_data_file(path.to_string_lossy().as_ref()).expect("落盘应成功");
    assert_eq!(std::fs::read(&path).expect("读回"), b"payload");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_flush_data_file_文件不存在时返回错误() {
    let path = std::env::temp_dir().join(format!(
        "molaunch-mapio-missing-{}-{}",
        std::process::id(),
        "nope"
    ));
    let _ = std::fs::remove_file(&path);
    assert!(
        flush_data_file(path.to_string_lossy().as_ref()).is_err(),
        "文件不存在时必须回报错误，不能当成刷成功"
    );
}
