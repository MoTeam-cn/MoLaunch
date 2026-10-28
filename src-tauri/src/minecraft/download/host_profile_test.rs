//! 主机记忆测试：EMA、目标合并、TTL 与主机识别

use super::{ema, host_of, merge, observe, suggest};
use std::time::Instant;

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
    // 1000 的老值 + 2000 的新样本，权重 0.3 -> 1300
    assert_eq!(ema(1000, 2000), 1300);
}

#[test]
fn test_merge_并发取大且受上下限约束() {
    let now = Instant::now();
    let first = merge(None, 3, 500, now);
    assert_eq!(first.target, 3);
    // 后续观测到更低的并发不缩小记忆（它只是起点）
    let second = merge(Some(first), 1, 500, now);
    assert_eq!(second.target, 3);
    // 更大的并发会提升记忆
    let third = merge(Some(second), 6, 500, now);
    assert_eq!(third.target, 6);
    // 超过上限收敛
    let over = merge(None, 99, 500, now);
    assert_eq!(over.target, 8);
}

#[test]
fn test_observe与suggest_跨调用生效() {
    let url = "https://profile-test.example.com/a.jar";
    assert_eq!(suggest(url), None);
    observe(url, 4, 1_000_000);
    assert_eq!(suggest(url), Some(4));
    // 另一个主机互不影响
    assert_eq!(
        suggest("https://other-profile-test.example.com/a.jar"),
        None
    );
}

#[test]
fn test_observe_吞吐做EMA() {
    let url = "https://ema-test.example.com/a.jar";
    observe(url, 2, 1000);
    observe(url, 2, 2000);
    let host = host_of(url).expect("主机");
    let profiles = super::profiles().lock().unwrap_or_else(|p| p.into_inner());
    assert_eq!(profiles.get(&host).expect("档案").throughput, 1300);
}
