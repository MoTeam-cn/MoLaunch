//! 引擎阶梯的纯逻辑测试（不下载、不联网）

use super::{feed, resume_key, FNV_OFFSET};
use crate::minecraft::download::types::DownloadTask;

fn task(id: &str, hash: Option<&str>) -> DownloadTask {
    DownloadTask {
        id: id.to_string(),
        urls: vec!["https://example.com/a.jar".to_string()],
        local_path: "/tmp/a.jar".to_string(),
        expected_size: 1024,
        expected_hash: hash.map(str::to_string),
    }
}

#[test]
fn test_resume_key_同输入稳定() {
    assert_eq!(
        resume_key(&task("t1", Some("sha1:abc")), 1024),
        resume_key(&task("t1", Some("sha1:abc")), 1024),
        "同任务同期望内容必须得到同一指纹"
    );
}

#[test]
fn test_resume_key_任务或期望内容变更则不同() {
    assert_ne!(
        resume_key(&task("t1", Some("sha1:abc")), 1024),
        resume_key(&task("t2", Some("sha1:abc")), 1024),
        "不同任务不能共用一次下载"
    );
    // 同名文件内容更新（哈希变了但大小相同）必须换指纹
    assert_ne!(
        resume_key(&task("t1", Some("sha1:abc")), 1024),
        resume_key(&task("t1", Some("sha1:def")), 1024)
    );
    assert_ne!(
        resume_key(&task("t1", Some("sha1:abc")), 1024),
        resume_key(&task("t1", Some("sha1:abc")), 2048)
    );
}

#[test]
fn test_resume_key_无期望哈希时不允许续传() {
    assert!(
        resume_key(&task("t1", None), 1024).is_none(),
        "没有期望哈希，拼出来的内容无从裁决，必须整份重下"
    );
    assert!(
        resume_key(&task("t1", Some("")), 1024).is_none(),
        "空哈希等同没有哈希"
    );
}

#[test]
fn test_resume_key_候选源不参与指纹() {
    let mut swapped = task("t1", Some("sha1:abc"));
    swapped.urls = vec![
        "https://mirror.example.com/a.jar".to_string(),
        "https://example.com/a.jar".to_string(),
    ];
    assert_eq!(
        resume_key(&task("t1", Some("sha1:abc")), 1024),
        resume_key(&swapped, 1024),
        "换下载源不改变期望内容，不能作废已下好的块"
    );
}

#[test]
fn test_feed_是确定性的FNV变体() {
    let mut hash = FNV_OFFSET;
    feed(&mut hash, b"abc");
    let first = hash;
    let mut again = FNV_OFFSET;
    feed(&mut again, b"abc");
    assert_eq!(first, again);
    // 换一个字节就应不同
    let mut other = FNV_OFFSET;
    feed(&mut other, b"abd");
    assert_ne!(first, other);
}
