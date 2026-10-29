//! 全局连接额度：所有下载任务共享一个总连接池
//!
//! 此前每个文件独立封顶（最多 12 条），文件并发一开就是 12×N 条连接，既压垮本机也容易
//! 被 CDN 判定为异常。这里把「最大线程数」当作**全局连接总预算**，按活跃文件数分摊，
//! 文件下完就把额度让给还在下的文件——不需要任何中心调度器，各文件每轮自己算一次即可。

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use sysinfo::{DiskExt, DiskKind, System, SystemExt};

/// 未设置时的总预算（与配置默认的最大线程数一致）
const DEFAULT_TOTAL: usize = 8;

/// 总预算上限：再大也没有意义，反而会拖慢彼此
const MAX_TOTAL: usize = 64;

/// 每个任务平均折算几条连接
///
/// 分片下载至少要 2 条连接才可能快于单流（1 条等于串行）；直接用 `max_threads` 当总预算的话，
/// 文件一多每个文件就只剩 1 条，批量安装反而退化。所以按「每任务 2 条」折算：
/// 默认 8 线程 → 16 条总预算，8 个文件并行时每个仍有 2 条。
const CONNECTIONS_PER_TASK: usize = 2;

static TOTAL: AtomicUsize = AtomicUsize::new(DEFAULT_TOTAL);
static ACTIVE_FILES: AtomicUsize = AtomicUsize::new(0);

/// 由「最大线程数」折算全局连接总预算
pub(crate) fn total_for_tasks(max_threads: usize) -> usize {
    max_threads
        .saturating_mul(CONNECTIONS_PER_TASK)
        .clamp(1, MAX_TOTAL)
}

/// 设置全局连接总预算（每个下载会话开始时由 `max_threads` 折算写入）
pub(crate) fn set_total(total: usize) {
    TOTAL.store(total.clamp(1, MAX_TOTAL), Ordering::Relaxed);
}

/// 当前全局连接总预算
pub(crate) fn total() -> usize {
    TOTAL.load(Ordering::Relaxed).clamp(1, MAX_TOTAL)
}

/// 当前正在并行下载的文件数
pub(crate) fn active_files() -> usize {
    ACTIVE_FILES.load(Ordering::Relaxed)
}

/// 一个文件的额度租约：打开时占一个活跃名额，`Drop` 时归还
///
/// 用 RAII 而不是手动加减：文件提前失败、被取消、panic 都不会漏归还额度。
pub(crate) struct FileLease;

impl FileLease {
    pub(crate) fn open() -> Self {
        ACTIVE_FILES.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for FileLease {
    fn drop(&mut self) {
        ACTIVE_FILES.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 当前每个文件应得的连接份额
///
/// `ceiling` 是单文件上限（用户设的分片数）：份额不会超过它，也不会低于 1
/// ——再拥挤也要给每个文件留一条连接，否则它会永远排不上。
pub(crate) fn per_file_share(ceiling: usize) -> usize {
    per_file_share_of(total(), active_files(), ceiling)
}

/// 纯函数版本：给定总预算、活跃文件数、单文件上限算出份额
pub(crate) fn per_file_share_of(total: usize, active: usize, ceiling: usize) -> usize {
    let active = active.max(1);
    (total / active).clamp(1, ceiling.max(1))
}

/// 磁盘类型（只关心对并发有影响的那一维）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiskCap {
    Solid,
    Rotational,
    Unknown,
}

/// 固态盘/未知盘的单文件连接上限
const SOLID_CONNECTIONS: usize = 8;

/// 机械盘：随机写要抢磁头，多条连接并行写只会互相拖慢
///
/// 不降到 1——绝大多数宽带的速率远低于一块机械盘的顺序写能力，只留一条连接
/// 会白白浪费带宽，反而更慢。
const ROTATIONAL_CONNECTIONS: usize = 2;

/// 磁盘类型对应的单文件并发
pub(crate) fn connections_for(kind: DiskCap) -> usize {
    match kind {
        DiskCap::Solid | DiskCap::Unknown => SOLID_CONNECTIONS,
        DiskCap::Rotational => ROTATIONAL_CONNECTIONS,
    }
}

/// 目标文件所在磁盘允许的单文件并发
///
/// 解析不出挂载点就按未知处理：宁可保守地用默认值，也不要凭空猜一个更小的上限。
pub(crate) fn disk_connection_cap(path: &Path) -> usize {
    connections_for(detect_disk_cap(path))
}

/// 找到包含该路径的挂载点（取最长的那个前缀）
fn detect_disk_cap(path: &Path) -> DiskCap {
    let mut system = System::new();
    system.refresh_disks_list();
    let mut best: Option<(usize, DiskCap)> = None;
    for disk in system.disks() {
        let mount = disk.mount_point();
        if !path.starts_with(mount) {
            continue;
        }
        let depth = mount.as_os_str().len();
        if best.is_none_or(|(best_depth, _)| depth > best_depth) {
            let kind = match disk.kind() {
                DiskKind::SSD => DiskCap::Solid,
                DiskKind::HDD => DiskCap::Rotational,
                DiskKind::Unknown(_) => DiskCap::Unknown,
            };
            best = Some((depth, kind));
        }
    }
    best.map(|(_, kind)| kind).unwrap_or(DiskCap::Unknown)
}

#[cfg(test)]
#[path = "budget_test.rs"]
mod budget_test;
