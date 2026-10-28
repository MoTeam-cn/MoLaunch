use super::{is_fresh, read_cache, write_cache, CacheResolver, CACHE_TTL};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn addr(last: u8) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, last)), 443)
}

#[test]
fn test_is_fresh_窗口内为真_超出窗口为假() {
    let at = Instant::now();
    assert!(is_fresh(at, at));
    assert!(is_fresh(at, at + CACHE_TTL - Duration::from_millis(1)));
    assert!(!is_fresh(at, at + CACHE_TTL));
    assert!(!is_fresh(at, at + CACHE_TTL + Duration::from_secs(1)));
}

#[test]
fn test_is_fresh_时间倒退按过期处理() {
    let now = Instant::now();
    // Instant 顺序异常时不能 panic，按过期处理即可
    let earlier = now + Duration::from_secs(10);
    assert!(!is_fresh(earlier, now));
}

#[test]
fn test_cache_命中与过期失效() {
    let resolver = CacheResolver::new();
    let now = Instant::now();
    assert!(read_cache(&resolver.cache, "cdn.example.com", now).is_none());
    write_cache(&resolver.cache, "cdn.example.com", &[addr(1), addr(2)], now);
    let hit = read_cache(&resolver.cache, "cdn.example.com", now).expect("应命中缓存");
    assert_eq!(hit, vec![addr(1), addr(2)]);
    // 其它 host 不受影响
    assert!(read_cache(&resolver.cache, "other.example.com", now).is_none());
    // 超出 TTL 后失效
    assert!(read_cache(
        &resolver.cache,
        "cdn.example.com",
        now + CACHE_TTL + Duration::from_secs(1)
    )
    .is_none());
}

#[test]
fn test_cache_覆盖写入生效() {
    let resolver = CacheResolver::new();
    let now = Instant::now();
    write_cache(&resolver.cache, "cdn.example.com", &[addr(1)], now);
    write_cache(&resolver.cache, "cdn.example.com", &[addr(9)], now);
    let hit = read_cache(&resolver.cache, "cdn.example.com", now).expect("应命中缓存");
    assert_eq!(hit, vec![addr(9)]);
}

#[test]
fn test_cache_可并发共享() {
    // Resolve 要求 Send + Sync：能被 Arc 包住并被多任务同时使用
    let resolver = Arc::new(CacheResolver::new());
    write_cache(
        &resolver.cache,
        "cdn.example.com",
        &[addr(3)],
        Instant::now(),
    );
    let cloned = Arc::clone(&resolver);
    let handle =
        std::thread::spawn(move || read_cache(&cloned.cache, "cdn.example.com", Instant::now()));
    assert!(handle.join().expect("线程不应 panic").is_some());
}
