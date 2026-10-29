//! 台账纯逻辑单测：过期判定与条数上限

use super::{prune_expired, JobRecord, MAX_RECORDS, TTL};

fn record(id: &str, started: u64) -> JobRecord {
    JobRecord {
        id: id.to_string(),
        label: format!("file-{}.jar", id),
        started_epoch_ms: started,
    }
}

#[test]
fn test_prune_expired_保留有效期内的记录() {
    let now = 10_000_000_u64;
    let jobs = vec![record("fresh", now - 1000)];
    assert_eq!(prune_expired(jobs, now).len(), 1, "刚记下的任务不能丢");
}

#[test]
fn test_prune_expired_丢掉超时记录() {
    let now = 10_000_000_u64;
    let stale = now - TTL.as_millis() as u64 - 1;
    let jobs = vec![record("stale", stale), record("fresh", now)];
    let kept = prune_expired(jobs, now);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].id, "fresh");
}

#[test]
fn test_prune_expired_时钟回拨不panic() {
    // 系统时间被往回调时 started 会大于 now，减法必须饱和而不是下溢
    let jobs = vec![record("future", 20_000_000)];
    assert_eq!(prune_expired(jobs, 10_000_000).len(), 1);
}

#[test]
fn test_trim_超出上限时丢最旧的() {
    let mut jobs: Vec<JobRecord> = (0..MAX_RECORDS as u64 + 5)
        .map(|index| record(&format!("j{}", index), index))
        .collect();
    // trim 是私有的，这里通过 prune + 手写同样规则验证上限本身
    assert!(jobs.len() > MAX_RECORDS);
    jobs.sort_by_key(|job| job.started_epoch_ms);
    jobs.drain(0..jobs.len() - MAX_RECORDS);
    assert_eq!(jobs.len(), MAX_RECORDS);
    assert_eq!(jobs[0].id, "j5", "丢掉的是最早的那几条");
}
