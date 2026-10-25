//! 后台任务主体：语言翻译 → class 文本 → 质量回修 → 重打包
//!
//! 每个阶段只返回 `StageResult`：完成 / 用户取消 / 致命错误。
//! 取消与失败都保留工作区与检查点供断点续传，成功打包后清理。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::AppHandle;

use crate::ai_core;
use crate::{log_error, log_info, log_warn};

use super::super::controller::clear_running;
use super::super::ledger::{ClassDecisionLedger, WorkGraph, WorkGraphSnapshot};
use super::super::memory::{self, TranslationMemory};
use super::super::mod_name;
use super::super::package;
use super::super::progress::{init_stage_weights, mark_stage_complete};
use super::super::repair;
use super::super::resume::{self, Checkpoint};
use super::super::status::{current_status, failed_snapshot, finish, update_status};
use super::super::translate_class;
use super::super::translate_lang;
use super::super::types::{LanguageSource, ProgressFn, RetryInfo, TaskSnapshot};
use super::prepare::Prepared;

/// 取消哨兵错误消息（各路由内部返回，据此判定为用户取消）
const CANCEL_MSG: &str = "任务已取消";

/// 路由错误归类：`None` 表示用户取消，`Some` 为致命错误
fn classify_error(error: String, cancel: &AtomicBool) -> Option<String> {
    if error == CANCEL_MSG || cancel.load(Ordering::Relaxed) {
        None
    } else {
        Some(error)
    }
}

/// 后台任务主体
#[allow(clippy::too_many_arguments)]
pub(crate) async fn run_task(
    app: AppHandle,
    prepared: Prepared,
    jar_path: PathBuf,
    config: ai_core::AiConfig,
    model: String,
    batch_size: usize,
    generate_mod_name: bool,
    repair_enabled: bool,
    class_text_enabled: bool,
    cancel_flag: Arc<AtomicBool>,
) {
    let Prepared {
        workspace,
        inspection,
        checkpoint,
    } = prepared;

    let has_language = inspection
        .language_sources
        .iter()
        .any(|source| source.required_count() > 0);
    if !has_language && inspection.class_candidates.is_empty() {
        finish(
            &app,
            failed_snapshot("translate", "未找到可翻译的 en_us 文本，或内容已全部翻译"),
        );
        cleanup(&workspace);
        clear_running();
        return;
    }

    let task_id = current_status().task_id;
    let mut checkpoint = checkpoint.unwrap_or_else(|| Checkpoint::fresh(task_id.clone()));
    init_stage_weights(repair_enabled, class_text_enabled);
    mark_completed_stages(&checkpoint.stage);

    let mut work_graph = restore_graph(&checkpoint, &task_id);
    let mut class_ledger = restore_ledger(&checkpoint);
    let mut completed_batches = checkpoint.completed_language_batches.clone();

    // 翻译记忆：全局共享（工作区根目录），任务结束统一 flush
    let memory_root = workspace
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let mut memory = TranslationMemory::load(memory::memory_path(&memory_root));

    let lang_progress = stage_progress(app.clone(), "language");
    let class_progress = stage_progress(app.clone(), "class");

    let mut cancelled = false;
    let mut fatal: Option<String> = None;

    // 1) 语言翻译（跳过已完成批次）
    if has_language {
        let mut pending = inspection.clone();
        pending.language_sources.retain(|source| {
            source.required_count() > 0 && !completed_batches.contains(&source.target_path)
        });
        if !pending.language_sources.is_empty() {
            update_status(&app, "language", 10.0, "开始翻译", None);
            let outcome = translate_lang::run_language_route(
                &workspace,
                &pending,
                &config,
                &model,
                batch_size,
                &mut memory,
                &mut work_graph,
                &cancel_flag,
                lang_progress.as_ref(),
            )
            .await;
            match outcome {
                Ok(()) => {
                    completed_batches.extend(
                        pending
                            .language_sources
                            .iter()
                            .map(|source| source.target_path.clone()),
                    );
                }
                Err(error) => match classify_error(error, &cancel_flag) {
                    Some(error) => {
                        log_error!("[ModTranslation] 语言翻译失败: {error}");
                        fatal = Some(error);
                    }
                    None => cancelled = true,
                },
            }
            if !cancelled && fatal.is_none() {
                checkpoint.completed_language_batches = completed_batches.clone();
                checkpoint.stage = "language".to_string();
                persist(&workspace, &mut checkpoint, &work_graph);
            }
        }
    }

    // 2) class 常量池文本翻译（断点续传：stage 为 class/repair 时已完成）
    let class_done = matches!(checkpoint.stage.as_str(), "class" | "repair");
    if !cancelled
        && fatal.is_none()
        && class_text_enabled
        && !inspection.class_candidates.is_empty()
        && !class_done
    {
        update_status(&app, "class", 0.0, "class 文本判定", None);
        let outcome = translate_class::run_class_route(
            &workspace,
            &inspection,
            &config,
            &model,
            &mut class_ledger,
            &cancel_flag,
            class_progress.as_ref(),
        )
        .await;
        match outcome {
            Ok(()) => {}
            Err(error) => match classify_error(error, &cancel_flag) {
                Some(error) => {
                    log_error!("[ModTranslation] class 文本翻译失败: {error}");
                    fatal = Some(error);
                }
                None => cancelled = true,
            },
        }
        if !cancelled && fatal.is_none() {
            save_class_state(&mut checkpoint, &class_ledger);
            checkpoint.stage = "class".to_string();
            persist(&workspace, &mut checkpoint, &work_graph);
        }
    }

    // 3) 质量回修兜底（断点续传：stage 为 repair 时已完成）
    let repair_done = checkpoint.stage == "repair";
    if !cancelled && fatal.is_none() && repair_enabled && !repair_done {
        let sources: Vec<&LanguageSource> = inspection
            .language_sources
            .iter()
            .filter(|source| source.required_count() > 0)
            .collect();
        if !sources.is_empty() {
            update_status(&app, "repair", 0.0, "质量复验中", None);
            let repair_progress: Arc<ProgressFn> = Arc::from(stage_progress(app.clone(), "repair"));
            let outcome = repair::run_repair_passes(
                &workspace,
                &sources,
                &mut work_graph,
                &config,
                &model,
                cancel_flag.clone(),
                repair_progress,
            )
            .await;
            match outcome {
                Ok(true) => {}
                Ok(false) => log_warn!("[ModTranslation] 质量回修存在残留问题，继续打包"),
                Err(error) => match classify_error(error, &cancel_flag) {
                    Some(error) => {
                        log_error!("[ModTranslation] 质量回修失败: {error}");
                        fatal = Some(error);
                    }
                    None => cancelled = true,
                },
            }
            if !cancelled && fatal.is_none() {
                checkpoint.stage = "repair".to_string();
                persist(&workspace, &mut checkpoint, &work_graph);
            }
        }
    }

    // 4) 结果处理
    let outcome = if cancelled {
        let current = current_status();
        Some(TaskSnapshot {
            status: "cancelled".to_string(),
            stage: "translate".to_string(),
            progress: current.progress,
            message: "任务已取消".to_string(),
            ..TaskSnapshot::new(String::new())
        })
    } else if let Some(error) = fatal {
        Some(failed_snapshot("translate", &error))
    } else {
        let name = generate_mod_name.then(|| {
            mod_name::resolve_mod_name(
                &inspection.project_names,
                &inspection.mod_ids,
                &inspection.original_filename,
                None,
            )
        });
        package::package(
            &app,
            &workspace,
            &jar_path,
            &inspection,
            name,
            &work_graph,
            &class_ledger,
        )
    };

    match outcome {
        Some(snapshot) => {
            let completed = snapshot.status == "completed";
            finish(&app, snapshot);
            if completed {
                cleanup(&workspace);
            } else {
                log_info!("[ModTranslation] 任务未完成，保留工作区供断点续传");
            }
        }
        None => cleanup(&workspace),
    }

    if let Err(error) = memory.flush() {
        log_warn!("[ModTranslation] 保存翻译记忆失败: {error}");
    }
    clear_running();
}

/// 每个阶段的进度回调
fn stage_progress(app: AppHandle, stage: &'static str) -> Box<ProgressFn> {
    Box::new(
        move |progress: f64, message: &str, retry: Option<RetryInfo>| {
            update_status(&app, stage, progress, message, retry)
        },
    )
}

/// 续传时把已完成阶段标记为 100%
fn mark_completed_stages(stage: &str) {
    match stage {
        "language" => mark_stage_complete("language"),
        "class" => {
            mark_stage_complete("language");
            mark_stage_complete("class");
        }
        "repair" => {
            mark_stage_complete("language");
            mark_stage_complete("class");
            mark_stage_complete("repair");
        }
        _ => {}
    }
}

fn restore_graph(checkpoint: &Checkpoint, task_id: &str) -> WorkGraph {
    checkpoint
        .work_graph
        .as_ref()
        .and_then(|value| serde_json::from_value::<WorkGraphSnapshot>(value.clone()).ok())
        .map(WorkGraph::from_snapshot)
        .unwrap_or_else(|| WorkGraph::new(task_id.to_string()))
}

fn restore_ledger(checkpoint: &Checkpoint) -> ClassDecisionLedger {
    ClassDecisionLedger {
        decisions: checkpoint.class_decisions.clone().into_iter().collect(),
        replaced_files: checkpoint.class_changed_files.clone(),
        replacement_count: checkpoint.class_replacement_count,
    }
}

fn save_class_state(checkpoint: &mut Checkpoint, ledger: &ClassDecisionLedger) {
    checkpoint.class_decisions = ledger.decisions.clone().into_iter().collect();
    checkpoint.class_exclusions = ledger.snapshot_exclusions();
    checkpoint.class_changed_files = ledger.replaced_files.clone();
    checkpoint.class_replacement_count = ledger.replacement_count;
}

/// 落盘检查点（失败只记录，不中断任务）
fn persist(workspace: &Path, checkpoint: &mut Checkpoint, work_graph: &WorkGraph) {
    checkpoint.work_graph = Some(serde_json::to_value(work_graph.snapshot()).unwrap_or_default());
    if let Err(error) = resume::save_checkpoint(workspace, checkpoint) {
        log_warn!("[ModTranslation] 保存检查点失败: {error}");
    }
}

/// 清理工作区（任务结束后释放缓存空间）
fn cleanup(workspace: &Path) {
    if let Err(error) = std::fs::remove_dir_all(workspace) {
        log_warn!("[ModTranslation] 清理工作区失败: {error}");
    }
}
