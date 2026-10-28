//! 引擎阶梯的纯逻辑测试（不下载、不联网）

use super::{feed, resume_key, sync_progress, FNV_OFFSET};
use crate::minecraft::download::types::{DownloadTask, GlobalProgress};
use std::sync::{Arc, Mutex as StdMutex};

fn task(id: &str, hash: Option<&str>) -> DownloadTask {
    DownloadTask {
        id: id.to_string(),
        urls: vec!["https://example.com/a.jar".to_string()],
        local_path: "/tmp/a.jar".to_string(),
        expected_size: 1024,
        expected_hash: hash.map(str::to_string),
    }
}

fn urls(list: &[&str]) -> Vec<String> {
    list.iter().map(|url| (*url).to_string()).collect()
}

#[test]
fn test_resume_key_同输入稳定() {
    let urls = urls(&["https://a/x.jar", "https://b/x.jar"]);
    let a = task("t1", Some("sha1:abc"));
    let b = task("t1", Some("sha1:abc"));
    assert_eq!(
        resume_key(&a, &urls, 1024),
        resume_key(&b, &urls, 1024),
        "同任务同来源必须得到同一指纹"
    );
}

#[test]
fn test_resume_key_任务变更则不同() {
    let urls = urls(&["https://a/x.jar"]);
    assert_ne!(
        resume_key(&task("t1", Some("sha1:abc")), &urls, 1024),
        resume_key(&task("t2", Some("sha1:abc")), &urls, 1024)
    );
    // 同名文件内容更新（哈希变了但大小相同）必须换指纹
    assert_ne!(
        resume_key(&task("t1", Some("sha1:abc")), &urls, 1024),
        resume_key(&task("t1", Some("sha1:def")), &urls, 1024)
    );
    assert_ne!(
        resume_key(&task("t1", None), &urls, 1024),
        resume_key(&task("t1", None), &urls, 2048)
    );
}

#[test]
fn test_resume_key_来源集合变更则不同() {
    let one = urls(&["https://a/x.jar"]);
    let two = urls(&["https://a/x.jar", "https://b/x.jar"]);
    assert_ne!(
        resume_key(&task("t1", None), &one, 1024),
        resume_key(&task("t1", None), &two, 1024)
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

#[test]
fn test_sync_progress_按当前可信字节对齐() {
    let progress = Arc::new(StdMutex::new(GlobalProgress {
        downloaded_bytes: 1000,
        ..Default::default()
    }));
    let slot = Some(progress.clone());
    let mut counted = 0_u64;
    // 第一轮跑出 300 字节 -> 全局 1300
    sync_progress(&slot, &mut counted, 300);
    assert_eq!(
        progress
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .downloaded_bytes,
        1300
    );
    // 作废重下使可信字节降到 120 -> 退回 180
    sync_progress(&slot, &mut counted, 120);
    assert_eq!(
        progress
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .downloaded_bytes,
        1120
    );
    // 重下补齐到 300 -> 再加 180
    sync_progress(&slot, &mut counted, 300);
    assert_eq!(
        progress
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .downloaded_bytes,
        1300
    );
    assert_eq!(counted, 300);
}
