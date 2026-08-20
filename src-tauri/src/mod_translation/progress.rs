//! 模组翻译：阶段进度管理 + AI 调用辅助（进度平滑、取消等待、超时配置）

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use super::types::{ProgressFn, RetryInfo, StageProgress};

/// 模组翻译 AI 调用超时（秒）：批量输出大、耗时长的场景覆盖全局默认 60s
pub(crate) const AI_TIMEOUT_SECS: u64 = 120;

/// 各阶段分进度（stage -> 0-100），用于计算总进度
static STAGE_PROGRESS: Mutex<BTreeMap<String, f64>> = Mutex::new(BTreeMap::new());
/// 阶段权重（任务启动时按启用开关设置，未启用阶段权重为 0）
static STAGE_WEIGHTS: Mutex<BTreeMap<String, f64>> = Mutex::new(BTreeMap::new());

/// 任务启动时重置阶段进度并设置权重（未启用阶段不设权重，总进度自动归一化）
pub(super) fn init_stage_weights(repair_enabled: bool, class_text_enabled: bool) {
    if let Ok(mut slot) = STAGE_PROGRESS.lock() {
        slot.clear();
    }
    let mut weights = BTreeMap::from([
        ("language".to_string(), 0.55),
        ("package".to_string(), 0.05),
    ]);
    if repair_enabled {
        weights.insert("repair".to_string(), 0.20);
    }
    if class_text_enabled {
        weights.insert("class".to_string(), 0.20);
    }
    if let Ok(mut slot) = STAGE_WEIGHTS.lock() {
        *slot = weights;
    }
}

/// 标记阶段已完成（断点续传补全进度用）
pub(super) fn mark_stage_complete(stage: &str) {
    if let Ok(mut slot) = STAGE_PROGRESS.lock() {
        slot.insert(stage.to_string(), 100.0);
    }
}

/// 写入某阶段分进度（update_status 推送进度时调用）
pub(super) fn set_stage_progress(stage: &str, progress: f64) {
    if let Ok(mut slot) = STAGE_PROGRESS.lock() {
        slot.insert(stage.to_string(), progress);
    }
}

/// 读取某阶段当前分进度（失败分支推送时取 max 避免进度回退）
pub(crate) fn current_stage_progress(stage: &str) -> f64 {
    STAGE_PROGRESS
        .lock()
        .map(|s| s.get(stage).copied().unwrap_or(0.0))
        .unwrap_or(0.0)
}

/// 等待取消信号（配合 tokio::select! 实现 AI 调用可即时取消）
pub(crate) async fn wait_cancel(cancel: &AtomicBool) {
    while !cancel.load(Ordering::Relaxed) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

/// 批次内平滑进度：AI 调用期间分进度从 start 平滑爬升到 cap（取消时结束）
/// `message` 为闭包，按当前进度动态生成消息（如批次内进度百分比）
/// 起始值取 max(start, 当前阶段已推送进度)，重试时不再回退
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
    let current = STAGE_PROGRESS
        .lock()
        .map(|s| s.get(stage).copied().unwrap_or(0.0))
        .unwrap_or(0.0);
    let mut p = start.max(current).min(cap);
    on_progress(p, &message(p), retry);
    while !cancel.load(Ordering::Relaxed) {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        if p < cap {
            p = (p + 0.5).min(cap);
            on_progress(p, &message(p), retry);
        }
    }
}

/// 按阶段权重加权计算总进度（0-100）
pub(super) fn compute_total_progress() -> f64 {
    let (weights, stages) = (
        STAGE_WEIGHTS.lock().unwrap_or_else(|e| e.into_inner()),
        STAGE_PROGRESS.lock().unwrap_or_else(|e| e.into_inner()),
    );
    let total_weight: f64 = weights.values().sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    weights
        .iter()
        .map(|(stage, w)| w * stages.get(stage).copied().unwrap_or(0.0))
        .sum::<f64>()
        / total_weight
}

/// 构建各阶段进度列表（前端分进度折叠区展示，按权重降序）
pub(super) fn build_stages() -> Vec<StageProgress> {
    let (weights, stages) = (
        STAGE_WEIGHTS.lock().unwrap_or_else(|e| e.into_inner()),
        STAGE_PROGRESS.lock().unwrap_or_else(|e| e.into_inner()),
    );
    let mut list: Vec<StageProgress> = weights
        .iter()
        .map(|(stage, weight)| StageProgress {
            stage: stage.clone(),
            weight: *weight,
            progress: stages.get(stage).copied().unwrap_or(0.0),
        })
        .collect();
    list.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    list
}
