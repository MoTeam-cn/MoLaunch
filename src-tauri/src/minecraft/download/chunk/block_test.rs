//! 分块写盘单测：写盘走阻塞线程池，偏移必须算对

use super::write_at;
use std::sync::{Arc, Mutex};

fn temp_file(tag: &str) -> (std::path::PathBuf, Arc<Mutex<std::fs::File>>) {
    let path = std::env::temp_dir().join(format!("molaunch-block-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_file(&path);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .expect("建临时文件");
    (path, Arc::new(Mutex::new(file)))
}

#[tokio::test]
async fn test_write_at_按绝对偏移写入() {
    let (path, file) = temp_file("offset");
    // 两段不重叠：0..5 与 5..10
    write_at(&file, 5, b"world".to_vec())
        .await
        .expect("写第二段");
    write_at(&file, 0, b"hello".to_vec())
        .await
        .expect("写第一段");

    let content = std::fs::read(&path).expect("读回");
    assert_eq!(content, b"helloworld", "偏移必须按绝对值定位，顺序无关");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn test_write_at_覆盖旧内容() {
    let (path, file) = temp_file("overwrite");
    write_at(&file, 0, b"0123456789".to_vec())
        .await
        .expect("先写满");
    write_at(&file, 3, b"AB".to_vec())
        .await
        .expect("再覆盖中间");

    let content = std::fs::read(&path).expect("读回");
    assert_eq!(content, b"012AB56789", "覆盖要落在正确的位置上");
    let _ = std::fs::remove_file(&path);
}
