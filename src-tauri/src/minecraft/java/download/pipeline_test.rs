//! 安装互斥单测：同一 component 必须串行，不同 component 不该互相排队

use super::lock_install;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_lock_install_同一component串行() {
    let inside = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    for _ in 0..4 {
        let inside = inside.clone();
        let peak = peak.clone();
        handles.push(tokio::spawn(async move {
            let _guard = lock_install("java-test-21").await;
            let now = inside.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(now, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(10)).await;
            inside.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for handle in handles {
        handle.await.expect("任务不应 panic");
    }
    assert_eq!(
        peak.load(Ordering::SeqCst),
        1,
        "同一 component 的安装必须串行，否则会互相覆盖文件"
    );
}

#[tokio::test]
async fn test_lock_install_不同component互不阻塞() {
    let held = lock_install("java-test-a").await;
    let other = tokio::time::timeout(Duration::from_secs(2), lock_install("java-test-b")).await;
    assert!(other.is_ok(), "不同 component 之间不该互相排队");
    drop(held);
    drop(other);
}
