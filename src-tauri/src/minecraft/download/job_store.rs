//! 下载任务台账：进程死掉之后，还知道「刚才有什么没跑完」
//!
//! 这里只解决一件事：**让中断可见**。
//!
//! 不做自动续传——重启后下载线程、HTTP 连接、位图锁全都没了，自动续跑需要把
//! 整个会话状态重建出来，那是另一个工程。台账能保证的是：下次启动时至少能说出
//! 「上次有 N 个任务没跑完」，而不是让它们在界面上静默消失。
//!
//! 文件放 `cache/download/active_jobs.json`，带版本号与 TTL：太久以前的记录不再
//! 可信（缓存与临时文件可能已被清理），读不懂就当成没有。

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::log_debug;

/// 台账文件（相对缓存根目录）
const STORE_PATH: &str = "download/active_jobs.json";

/// 落盘格式版本
const STORE_VERSION: u32 = 1;

/// 记录上限：整合包几百个文件，再多说明台账本身出了问题
const MAX_RECORDS: usize = 512;

/// 记录有效期：超过这个时间的残留不再有意义
const TTL: Duration = Duration::from_secs(2 * 60 * 60);

/// 一条在跑的任务
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JobRecord {
    pub(crate) id: String,
    /// 给人看的标识（通常是目标路径）
    pub(crate) label: String,
    pub(crate) started_epoch_ms: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Store {
    version: u32,
    jobs: Vec<JobRecord>,
}

fn state() -> &'static Mutex<Option<Vec<JobRecord>>> {
    static JOBS: OnceLock<Mutex<Option<Vec<JobRecord>>>> = OnceLock::new();
    JOBS.get_or_init(|| Mutex::new(None))
}

/// 登记一个开始的任务（同一 id 重复登记只留最早那条）
pub(crate) fn begin(id: &str, label: &str) {
    let now = unix_millis();
    let mut guard = state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let jobs = guard.get_or_insert_with(load_or_default);
    if jobs.iter().any(|job| job.id == id) {
        return;
    }
    jobs.push(JobRecord {
        id: id.to_string(),
        label: label.to_string(),
        started_epoch_ms: now,
    });
    trim(jobs);
    persist(jobs);
}

/// 注销一个结束的任务（成功、失败、取消都算结束）
pub(crate) fn finish(id: &str) {
    let mut guard = state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(jobs) = guard.as_mut() else {
        return;
    };
    let before = jobs.len();
    jobs.retain(|job| job.id != id);
    if jobs.len() != before {
        persist(jobs);
    }
}

/// 取走上次进程留下的未过期记录，并清空台账
///
/// 取走而不是只读：这些任务在当前进程里已经不存在了，留着只会被反复报告。
pub(crate) fn recover() -> Vec<JobRecord> {
    let now = unix_millis();
    let mut guard = state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let jobs = guard.get_or_insert_with(load_or_default);
    let leftover = prune_expired(std::mem::take(jobs), now);
    persist(jobs);
    log_debug!("[JobStore] 恢复{} 条上次未完成的记录", leftover.len());
    leftover
}

/// 丢掉过期记录并把条数压到上限以内（保留最近的）
fn prune_expired(jobs: Vec<JobRecord>, now_ms: u64) -> Vec<JobRecord> {
    let ttl_ms = TTL.as_millis() as u64;
    jobs.into_iter()
        .filter(|job| now_ms.saturating_sub(job.started_epoch_ms) <= ttl_ms)
        .collect()
}

fn trim(jobs: &mut Vec<JobRecord>) {
    if jobs.len() <= MAX_RECORDS {
        return;
    }
    jobs.sort_by_key(|job| job.started_epoch_ms);
    let overflow = jobs.len() - MAX_RECORDS;
    jobs.drain(0..overflow);
}

fn load_or_default() -> Vec<JobRecord> {
    let Some(path) = store_path() else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    match serde_json::from_str::<Store>(&text) {
        Ok(store) if store.version == STORE_VERSION => prune_expired(store.jobs, unix_millis()),
        Ok(_) => {
            log_debug!("[JobStore] 台账版本不认识，当成空的");
            Vec::new()
        }
        Err(e) => {
            log_debug!("[JobStore] 台账解析失败，当成空的: {}", e);
            Vec::new()
        }
    }
}

fn persist(jobs: &[JobRecord]) {
    let Some(path) = store_path() else {
        return;
    };
    let store = Store {
        version: STORE_VERSION,
        jobs: jobs.to_vec(),
    };
    let Ok(text) = serde_json::to_string(&store) else {
        return;
    };
    if let Err(e) = crate::utils::fs::write_atomic(&path, text.as_bytes()) {
        crate::log_error!("[JobStore] 任务台账落盘失败，未完成任务将无法续传: {}", e);
    }
}

fn store_path() -> Option<std::path::PathBuf> {
    Some(crate::storage::cache::Cache::instance().path(STORE_PATH))
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 作用域内算「一个在跑的任务」：离开作用域自动注销
///
/// 用 RAII 而不是在每个返回点手写 finish：下载路径有一堆提前返回与 `?`，
/// 漏一个就会在台账里留下永远不消失的幽灵任务。
pub(crate) struct JobGuard {
    id: String,
}

impl JobGuard {
    pub(crate) fn new(id: &str, label: &str) -> Self {
        begin(id, label);
        Self { id: id.to_string() }
    }
}

impl Drop for JobGuard {
    fn drop(&mut self) {
        finish(&self.id);
    }
}

#[cfg(test)]
#[path = "job_store_test.rs"]
mod job_store_test;
