//! 并行下载单测：数据落盘

use super::flush_data_file;
use std::path::PathBuf;

fn temp_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("molaunch-multi-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    dir.join(name)
}

#[test]
fn test_flush_data_file_写入后落盘成功且内容不变() {
    let path = temp_file("ok.bin");
    std::fs::write(&path, b"payload").expect("写测试文件");
    flush_data_file(path.to_str().expect("路径可转 utf8")).expect("落盘应成功");
    assert_eq!(std::fs::read(&path).expect("读回"), b"payload");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_flush_data_file_文件不存在时返回错误() {
    let path = temp_file("missing.bin");
    let _ = std::fs::remove_file(&path);
    let result = flush_data_file(path.to_str().expect("路径可转 utf8"));
    assert!(result.is_err(), "文件不存在必须报错，不能静默成功");
}
