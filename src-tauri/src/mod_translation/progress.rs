//! 模组翻译：阶段进度管理 + AI 调用辅助（进度平滑、取消等待、超时配置）
//!
//! 总进度 = 各阶段分进度按权重加权；未启用的阶段不设权重，因此自动归一化。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use super::types::{ProgressFn, RetryInfo, StageProgress};

/// 模组翻译 AI 调用超时（秒）：批量输出大、耗时长，覆盖全局默认 60s
pub(crate) const AI_TIMEOUT_SECS: u64 = 120;
/// 进度平滑间隔
const TICK: std::time::Duration = std::time::Duration::from_millis(200);
/// 取消轮询间隔
const CANCEL_POLL: std::time::Duration = std::time::Duration::from_millis(100);
/// 每次 tick 的爬升量
const STEP: f64 = 0.5;
/// 语言阶段权重
const LANGUAGE_WEIGHT: f64 = 0.55;
/// 打包阶段权重
const PACKAGE_WEIGHT: f64 = 0.05;
/// 质量回修阶段权重
const REPAIR_WEIGHT: f64 = 0.20;
/// class 阶段权重
const CLASS_WEIGHT: f64 = 0.20;

static STAGE_PROGRESS: Mutex<BTreeMap<String, f64>> = Mutex::new(BTreeMap::new());
static STAGE_WEIGHTS: Mutex<BTreeMap<String, f64>> = Mutex::new(BTreeMap::new());

/// 中毒的锁也能继续用（进度只是展示数据）
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

/// 任务启动时重置阶段进度并设置权重
pub(super) fn init_stage_weights(repair_enabled: bool, class_text_enabled: bool) {
    lock(&STAGE_PROGRESS).clear();

    let mut weights = BTreeMap::from([
        ("language".to_string(), LANGUAGE_WEIGHT),
        ("package".to_string(), PACKAGE_WEIGHT),
    ]);
    if repair_enabled {
        weights.insert("repair".to_string(), REPAIR_WEIGHT);
    }
    if class_text_enabled {
        weights.insert("class".to_string(), CLASS_WEIGHT);
    }
    *lock(&STAGE_WEIGHTS) = weights;
}

/// 标记阶段已完成（断点续传补全进度用）
pub(super) fn mark_stage_complete(stage: &str) {
    lock(&STAGE_PROGRESS).insert(stage.to_string(), 100.0);
}

/// 写入某阶段分进度
pub(super) fn set_stage_progress(stage: &str, progress: f64) {
    lock(&STAGE_PROGRESS).insert(stage.to_string(), progress);
}

/// 读取某阶段当前分进度（失败分支取 max 避免进度回退）
pub(crate) fn current_stage_progress(stage: &str) -> f64 {
    lock(&STAGE_PROGRESS).get(stage).copied().unwrap_or(0.0)
}

/// 等待取消信号（配合 tokio::select! 实现 AI 调用即时取消）
pub(crate) async fn wait_cancel(cancel: &AtomicBool) {
    while !cancel.load(Ordering::Relaxed) {
        tokio::time::sleep(CANCEL_POLL).await;
    }
}

/// 批次内平滑进度：从 start 爬到 cap，取消时结束
pub(crate) async fn smooth_progress<F>(
    stage: &str,
    start: f64,
    cap: f64,
    cancel: &AtomicBool,
    on_progress: &ProgressFn,
    message: F,
    retry: Option<RetryInfo>,
) where
    F: Fn(f64) -> String + Send + Sync + 'static,
{
    let mut value = start.max(current_stage_progress(stage)).min(cap);
    on_progress(value, &message(value), retry);

    while !cancel.load(Ordering::Relaxed) {
        tokio::time::sleep(TICK).await;
        if value >= cap {
            continue;
        }
        value = (value + STEP).min(cap);
        on_progress(value, &message(value), retry);
    }
}

/// 按阶段权重加权计算总进度（0-100）
pub(super) fn compute_total_progress() -> f64 {
    let weights = lock(&STAGE_WEIGHTS);
    let stages = lock(&STAGE_PROGRESS);
    let total_weight: f64 = weights.values().sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    let weighted: f64 = weights
        .iter()
        .map(|(stage, weight)| weight * stages.get(stage).copied().unwrap_or(0.0))
        .sum();
    weighted / total_weight
}

/// 各阶段进度列表（前端分进度折叠区，按权重降序）
pub(super) fn build_stages() -> Vec<StageProgress> {
    let weights = lock(&STAGE_WEIGHTS);
    let stages = lock(&STAGE_PROGRESS);
    let mut list: Vec<StageProgress> = weights
        .iter()
        .map(|(stage, weight)| StageProgress {
            stage: stage.clone(),
            weight: *weight,
            progress: stages.get(stage).copied().unwrap_or(0.0),
        })
        .collect();
    list.sort_by(|left, right| right.weight.total_cmp(&left.weight));
    list
}
