//! 模组翻译：任务编排入口（分析 → 翻译 → 重打包）
//!
//! 单任务模型：同一时间仅允许一个翻译任务；分析结果在工作区缓存，
//! 启动翻译时若路径一致则复用，否则重新解包分析。

pub mod analyze;
pub mod class;
pub mod error;
pub mod jar;
pub mod json_value;
pub mod lang;
pub mod ledger;
pub mod memory;
pub mod mod_name;
pub mod package;
pub mod prompt;
pub mod quality;
pub mod repair;
pub mod resume;
pub mod task;
pub mod translate_class;
pub mod translate_lang;
pub mod types;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter};

use crate::ai_core;

use self::resume::Checkpoint;
use self::types::{
    AnalyzeParams, AnalyzeResult, JarInspection, RetryInfo, SourceSummary, StageProgress,
    StartParams, TaskSnapshot,
};

/// 进度事件名（前端经 useTauriEvent 订阅）
pub const EVENT_NAME: &str = "mod-translation-event";

/// 已就绪的分析结果（analyze 与 start 之间传递工作区）
#[derive(Clone)]
pub(crate) struct Prepared {
    pub workspace: PathBuf,
    pub inspection: JarInspection,
    /// 断点续传检查点（复用续传工作区时存在）
    pub checkpoint: Option<Checkpoint>,
}

/// 进行中的任务（cancel_flag 置位后翻译循环尽快中止）
struct RunningTask {
    cancel_flag: Arc<AtomicBool>,
}

static PREPARED: Mutex<Option<Prepared>> = Mutex::new(None);
static RUNNING: Mutex<Option<RunningTask>> = Mutex::new(None);
static STATUS: Mutex<Option<TaskSnapshot>> = Mutex::new(None);
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

/// 按阶段权重加权计算总进度（0-100）
fn compute_total_progress() -> f64 {
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
fn build_stages() -> Vec<StageProgress> {
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

/// 分析 JAR：解包 → 探测加载器 → 汇总语言源
pub async fn analyze_jar(params: AnalyzeParams) -> Result<AnalyzeResult, String> {
    ensure_idle()?;
    let jar_path = PathBuf::from(params.jar_path);
    if !jar_path.is_file() {
        return Err(format!("文件不存在: {}", jar_path.display()));
    }
    let prepared = task::prepare(&jar_path)?;
    if let Ok(mut slot) = PREPARED.lock() {
        *slot = Some(prepared.clone());
    }
    Ok(to_result(&prepared.inspection))
}

/// 启动翻译任务（后台执行，立即返回任务快照）
pub async fn start_task(app: AppHandle, params: StartParams) -> Result<TaskSnapshot, String> {
    ensure_idle()?;
    let jar_path = PathBuf::from(params.jar_path);
    if !jar_path.is_file() {
        return Err(format!("文件不存在: {}", jar_path.display()));
    }

    // 路径一致时复用已分析的工作区，否则重新解包
    let prepared = match PREPARED.lock().ok().and_then(|g| g.clone()) {
        Some(p) if p.inspection.input_path == jar_path => p,
        _ => task::prepare(&jar_path)?,
    };

    let config = ai_core::load_config_async().await;
    if config.base_url.trim().is_empty() {
        return Err(
            "未配置 AI 服务地址，请先在「实验性 → AI 设置」中配置本地 OpenAI 兼容服务".to_string(),
        );
    }
    let model = if params.model.trim().is_empty() {
        config.resolve_model(None)
    } else {
        params.model.trim().to_string()
    };
    if model.is_empty() {
        return Err("未启用任何模型，请先在「实验性 → AI 设置」中加载并启用模型".to_string());
    }

    let task_id = format!(
        "mod-translation-{}",
        chrono::Local::now().timestamp_millis()
    );
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let snapshot = TaskSnapshot::new(task_id);
    if let Ok(mut slot) = RUNNING.lock() {
        *slot = Some(RunningTask {
            cancel_flag: cancel_flag.clone(),
        });
    }
    store_status(&snapshot);

    let app_for_task = app.clone();
    tauri::async_runtime::spawn(async move {
        task::run_task(
            app_for_task,
            prepared,
            jar_path,
            config,
            model,
            params.batch_size as usize,
            params.generate_mod_name,
            params.repair_enabled,
            params.class_text_enabled,
            cancel_flag,
        )
        .await;
    });
    Ok(snapshot)
}

/// 取消当前任务（置位取消信号，翻译循环尽快中止）
pub fn cancel_task() -> Result<(), String> {
    let slot = RUNNING.lock().unwrap_or_else(|e| e.into_inner());
    match slot.as_ref() {
        Some(task) => {
            task.cancel_flag.store(true, Ordering::Relaxed);
            Ok(())
        }
        None => Err("当前没有进行中的翻译任务".to_string()),
    }
}

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

/// 汇总为前端展示结果
fn to_result(inspection: &JarInspection) -> AnalyzeResult {
    AnalyzeResult {
        filename: inspection.original_filename.clone(),
        loader: inspection.loader.as_str().to_string(),
        mod_ids: inspection.mod_ids.clone(),
        project_names: inspection.project_names.clone(),
        version: inspection.version.clone(),
        signed: inspection.signed,
        sources: inspection
            .language_sources
            .iter()
            .map(|s| SourceSummary {
                kind: s.kind.as_str().to_string(),
                namespace: s.namespace.clone(),
                source_path: s.source_path.clone(),
                target_path: s.target_path.clone(),
                entries: s.required_count(),
            })
            .collect(),
        total_entries: inspection
            .language_sources
            .iter()
            .map(|s| s.required_count())
            .sum(),
        class_candidates: inspection.class_candidates.clone(),
        quote: inspection.quote.clone(),
        coverage: inspection.coverage.clone(),
        mod_name: None,
        existing_chinese: inspection.existing_chinese.clone(),
        warnings: inspection.warnings.clone(),
    }
}

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
    if let Ok(mut slot) = STAGE_PROGRESS.lock() {
        slot.insert(stage.to_string(), progress);
    }
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

fn store_status(snapshot: &TaskSnapshot) {
    if let Ok(mut slot) = STATUS.lock() {
        *slot = Some(snapshot.clone());
    }
}

pub(super) fn clear_running() {
    if let Ok(mut slot) = RUNNING.lock() {
        *slot = None;
    }
}

/// 有任务进行中时拒绝新的分析/启动
fn ensure_idle() -> Result<(), String> {
    if RUNNING.lock().unwrap_or_else(|e| e.into_inner()).is_some() {
        return Err("已有翻译任务进行中，请等待完成或取消".to_string());
    }
    Ok(())
}
