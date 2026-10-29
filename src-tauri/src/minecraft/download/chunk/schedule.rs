//! 源调度决策：初始分配、扩容、收缩、判死
//!
//! 全是纯函数：引擎只负责收集每轮的实测速率，做决定的地方集中在这里，方便单测。

/// 单文件同时最多几条连接
pub(crate) const MAX_WORKERS_PER_FILE: usize = 12;

/// 明显快于全局均值才给它加连接（避免抖动导致连接数乱涨）
const GROW_FACTOR: f64 = 1.2;

/// 初始 worker 分配：只有一条源就全给它，多条源先各给一条
pub(crate) fn initial_workers(source_count: usize, budget: usize) -> Vec<usize> {
    if source_count == 0 {
        return Vec::new();
    }
    let budget = budget.max(1);
    if source_count == 1 {
        return vec![budget];
    }
    let mut counts = vec![0_usize; source_count];
    for slot in counts.iter_mut().take(source_count.min(budget)) {
        *slot = 1;
    }
    counts
}

/// 是否该给某条源再加一条连接
pub(crate) fn should_grow(rate: u64, avg: u64, active_total: usize, budget: usize) -> bool {
    if active_total >= budget {
        return false;
    }
    if avg == 0 {
        return rate > 0;
    }
    (rate as f64) >= (avg as f64) * GROW_FACTOR
}

/// 是否该给某条源减少一条连接
///
/// 只在该源明显低于均值（不足一半）且当前多于 1 条时才减：单条连接永远保底，
/// 否则一条慢源会被自己饿死，反而更慢。
pub(crate) fn should_shrink(rate: u64, avg: u64, current: usize) -> bool {
    current > 1 && avg > 0 && rate.saturating_mul(2) < avg
}

/// 该源本轮是否该判死
///
/// 只有「本轮零产出」才算候选，且必须同时满足：还有别的活源、整体连接数不止一条
/// ——唯一活源即使零产出也要留着，否则永远拿不到进展。
pub(crate) fn should_retire(rate: u64, alive: usize, active_total: usize) -> bool {
    rate == 0 && alive > 1 && active_total > 1
}

/// 一个采样窗口的并发调整计划（纯决策，不碰 IO）
pub(crate) struct WindowPlan {
    /// 要各减一条连接的源（sources 下标）
    pub(crate) shrink: Vec<usize>,
    /// 要加一条连接的源
    pub(crate) grow: Option<usize>,
}

/// 探针扩容的间隔（窗口数）
///
/// 所有源速度都差不多时，「明显快于均值」永远选不出赢家，于是闲置额度一直没人用。
/// 每隔几个窗口按速率挑一条试加一条连接：加对了下一窗口继续，加错了下一窗口被收缩收回。
const PROBE_INTERVAL_WINDOWS: u64 = 5;

/// 算出这个窗口的调整计划
///
/// `items` 是 (sources 下标, 本窗口速率, 当前目标连接数) 三元组，只包含还活着的源。
/// `throttled` 表示上一窗口又被限流过：这时只许退、不许进。
pub(crate) fn plan_window(
    items: &[(usize, u64, usize)],
    avg: u64,
    share: usize,
    throttled: bool,
    window: u64,
) -> WindowPlan {
    let total_target: usize = items.iter().map(|(_, _, target)| *target).sum();
    let mut shrink = Vec::new();
    for (index, rate, current) in items {
        if *current > 1
            && (throttled || should_shrink(*rate, avg, *current) || total_target > share)
        {
            shrink.push(*index);
        }
    }
    if throttled {
        return WindowPlan { shrink, grow: None };
    }
    let grow = items
        .iter()
        .max_by_key(|(_, rate, _)| *rate)
        .and_then(|(index, rate, _)| {
            let by_rate = should_grow(*rate, avg, total_target, share);
            let by_probe = window.is_multiple_of(PROBE_INTERVAL_WINDOWS)
                && total_target < share
                && *rate > 0;
            if by_rate || by_probe {
                Some(*index)
            } else {
                None
            }
        });
    WindowPlan { shrink, grow }
}

#[cfg(test)]
#[path = "schedule_test.rs"]
mod schedule_test;
