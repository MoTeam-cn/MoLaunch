//! 模组翻译：任务状态快照与进度事件推送

use std::sync::Mutex;

use tauri::{AppHandle, Emitter};

use super::progress::{build_stages, compute_total_progress, set_stage_progress};
use super::types::{RetryInfo, TaskSnapshot};

/// 进度事件名（前端经 useTauriEvent 订阅）
pub const EVENT_NAME: &str = "mod-translation-event";

/// 当前任务状态快照（无任务时返回 idle）
static STATUS: Mutex<Option<TaskSnapshot>> = Mutex::new(None);

/// 当前任务状态快照（无任务时返回 idle）
pub fn current_status() -> TaskSnapshot {
    STATUS
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| TaskSnapshot {
            task_id: String::new(),
            status: "idle".to_string(),
            stage: String::new(),
            progress: 0.0,
            stage_progress: 0.0,
            retry: None,
            stages: Vec::new(),
            message: String::new(),
            output_path: None,
            error: None,
            mod_name: None,
            report: None,
        })
}

/// 失败终态快照（task_id 为空，finish 时补全）
pub(super) fn failed_snapshot(stage: &str, error: &str) -> TaskSnapshot {
    TaskSnapshot {
        status: "failed".to_string(),
        stage: stage.to_string(),
        progress: 0.0,
        message: "任务失败".to_string(),
        error: Some(error.to_string()),
        ..TaskSnapshot::new(String::new())
    }
}

/// 更新状态并向前端 emit 进度事件（progress 为当前阶段分进度，总进度按权重计算）
pub(super) fn update_status(
    app: &AppHandle,
    stage: &str,
    progress: f64,
    message: &str,
    retry: Option<RetryInfo>,
) {
    set_stage_progress(stage, progress);
    let mut snapshot = current_status();
    snapshot.stage = stage.to_string();
    snapshot.stage_progress = progress;
    snapshot.progress = compute_total_progress();
    snapshot.retry = retry;
    snapshot.stages = build_stages();
    snapshot.message = message.to_string();
    store_status(&snapshot);
    let _ = app.emit(EVENT_NAME, &snapshot);
}

/// 终态：写入状态 + emit 事件（task_id 为空时从当前状态补全）
pub(super) fn finish(app: &AppHandle, mut snapshot: TaskSnapshot) {
    if snapshot.task_id.is_empty() {
        snapshot.task_id = current_status().task_id;
    }
    store_status(&snapshot);
    let _ = app.emit(EVENT_NAME, &snapshot);
}

/// 写入状态快照（命令入口与终态共用）
pub(super) fn store_status(snapshot: &TaskSnapshot) {
    if let Ok(mut slot) = STATUS.lock() {
        *slot = Some(snapshot.clone());
    }
}
