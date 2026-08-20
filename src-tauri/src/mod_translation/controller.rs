//! 模组翻译：命令入口（分析 / 启动 / 取消）

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::AppHandle;

use crate::ai_core;

use super::progress::{build_stages, init_stage_weights};
use super::status::store_status;
use super::task::{self, Prepared};
use super::types::{
    AnalyzeParams, AnalyzeResult, JarInspection, SourceSummary, StartParams, TaskSnapshot,
};

/// 进行中的任务（cancel_flag 置位后翻译循环尽快中止）
struct RunningTask {
    cancel_flag: Arc<AtomicBool>,
}

/// 已就绪的分析结果（analyze 与 start 之间传递工作区）
static PREPARED: Mutex<Option<Prepared>> = Mutex::new(None);
/// 进行中的任务（cancel_flag 置位后翻译循环尽快中止）
static RUNNING: Mutex<Option<RunningTask>> = Mutex::new(None);

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
    // 任务启动即初始化阶段权重与进度列表，前端立即能看到各阶段（避免事件到达前无子阶段）
    init_stage_weights(params.repair_enabled, params.class_text_enabled);
    let mut snapshot = TaskSnapshot::new(task_id);
    snapshot.stages = build_stages();
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

/// 任务结束清理运行标记（成功/失败/取消均调用）
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
