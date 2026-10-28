//! 主机记忆测试：主机识别、EMA、目标合并、落盘往返与清理

use super::{
    ema, host_of, load_from, merge, observe, save_to, suggest, PersistedProfile, Store,
    STORE_VERSION, TTL,
};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn temp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("molaunch-hostprofile-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    dir.join(name)
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[test]
fn test_host_of_提取小写主机() {
    assert_eq!(
        host_of("https://BMCLAPI2.bangbang93.com/maven/a.jar").as_deref(),
        Some("bmclapi2.bangbang93.com")
    );
    assert_eq!(host_of("not a url"), None);
}

#[test]
fn test_ema_首个样本直接采用() {
    assert_eq!(ema(0, 1000), 1000);
}

#[test]
fn test_ema_新样本按权重混合() {
    assert_eq!(ema(1000, 2000), 1300);
}

#[test]
fn test_merge_并发取大且受上下限约束() {
    let now = std::time::Instant::now();
    let first = merge(None, 3, 500, now);
    assert_eq!(first.target, 3);
    let second = merge(Some(first), 1, 500, now);
    assert_eq!(second.target, 3, "更低的观测不缩小记忆");
    let third = merge(Some(second), 6, 500, now);
    assert_eq!(third.target, 6);
    assert_eq!(merge(None, 99, 500, now).target, 8, "超过上限收敛");
}

#[test]
fn test_observe与suggest_跨调用生效() {
    let url = "https://profile-test.example.com/a.jar";
    observe(url, 4, 1_000_000);
    assert_eq!(suggest(url), Some(4));
    assert_eq!(
        suggest("https://other-profile-test.example.com/a.jar"),
        None
    );
}

#[test]
fn test_observe_零吞吐不构成经验() {
    let url = "https://zero-throughput.example.com/a.jar";
    observe(url, 4, 0);
    assert_eq!(suggest(url), None, "一个字节都没下成不应写入记忆");
}

#[test]
fn test_store_往返保留并发与吞吐() {
    let path = temp_path("roundtrip.json");
    let mut profiles = std::collections::HashMap::new();
    profiles.insert(
        "cdn.example.com".to_string(),
        PersistedProfile {
            target: 4,
            throughput: 1_234_567,
            learned_epoch_ms: now_millis(),
        },
    );
    let store = Store {
        version: STORE_VERSION,
        profiles,
    };
    save_to(&path, &store).expect("写盘");

    let runtime = load_from(&path).expect("读盘").into_runtime();
    let profile = runtime.get("cdn.example.com").expect("应保留");
    assert_eq!(profile.target, 4);
    assert_eq!(profile.throughput, 1_234_567);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_store_版本不符则整体忽略() {
    let path = temp_path("version.json");
    let mut profiles = std::collections::HashMap::new();
    profiles.insert(
        "cdn.example.com".to_string(),
        PersistedProfile {
            target: 4,
            throughput: 1,
            learned_epoch_ms: now_millis(),
        },
    );
    let store = Store {
        version: STORE_VERSION + 1,
        profiles,
    };
    save_to(&path, &store).expect("写盘");
    let runtime = load_from(&path).expect("读盘").into_runtime();
    assert!(runtime.is_empty(), "版本不认识就当没有，不猜");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_store_过期条目被丢弃() {
    let path = temp_path("expired.json");
    let stale = now_millis() - TTL.as_millis() as u64 - Duration::from_secs(60).as_millis() as u64;
    let mut profiles = std::collections::HashMap::new();
    profiles.insert(
        "old.example.com".to_string(),
        PersistedProfile {
            target: 6,
            throughput: 1,
            learned_epoch_ms: stale,
        },
    );
    profiles.insert(
        "fresh.example.com".to_string(),
        PersistedProfile {
            target: 2,
            throughput: 1,
            learned_epoch_ms: now_millis(),
        },
    );
    let store = Store {
        version: STORE_VERSION,
        profiles,
    };
    save_to(&path, &store).expect("写盘");
    let runtime = load_from(&path).expect("读盘").into_runtime();
    assert!(!runtime.contains_key("old.example.com"), "过期条目应丢弃");
    assert!(runtime.contains_key("fresh.example.com"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_store_损坏文件读盘报错() {
    let path = temp_path("broken.json");
    std::fs::write(&path, b"not json").expect("写脏数据");
    assert!(load_from(&path).is_err());
    let _ = std::fs::remove_file(&path);
}
