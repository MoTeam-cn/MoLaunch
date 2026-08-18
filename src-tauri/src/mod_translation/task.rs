//! 模组翻译：任务编排（多阶段翻译 + 断点续传 + 质量回修 + 重打包）

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::AppHandle;

use crate::ai_core;
use crate::storage::cache::Cache;
use crate::{log_error, log_info, log_warn};

use super::analyze;
use super::jar;
use super::ledger::{ClassDecision, ClassDecisionLedger, WorkGraph, WorkGraphSnapshot};
use super::memory::{self, TranslationMemory};
use super::mod_name;
use super::package;
use super::repair;
use super::resume::{self, Checkpoint};
use super::translate_class;
use super::translate_lang;
use super::types::{LanguageSource, TaskSnapshot};
use super::{failed_snapshot, finish, update_status, Prepared};

/// 缓存工作区根目录（`.Molaunch/cache/mod-translation`）
const WORKSPACE_ROOT: &str = "mod-translation";
/// 续传身份标识（写入/匹配 resume marker 用）
const RESUME_IDENTITY: &str = "mod-translator-v1";
/// 取消哨兵错误消息（各路由内部返回，据此判定为用户取消）
const CANCEL_MSG: &str = "任务已取消";

/// 后台任务主体：语言翻译 → 质量回修 → class 文本 → 重打包。
/// 取消/失败保留工作区与检查点供断点续传，成功完成后清理。
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_task(
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
        .any(|s| s.required_count() > 0);
    if !has_language && inspection.class_candidates.is_empty() {
        finish(
            &app,
            failed_snapshot("translate", "未找到可翻译的 en_us 文本，或内容已全部翻译"),
        );
        cleanup(&workspace);
        super::clear_running();
        return;
    }

    let task_id = super::current_status().task_id;

    // 断点续传：恢复检查点、工作图与 class 账本
    let mut checkpoint = checkpoint.unwrap_or_else(|| Checkpoint::fresh(task_id.clone()));
    super::init_stage_weights(repair_enabled, class_text_enabled);

    // 断点续传：补全已完成阶段进度（前端分进度折叠区显示 100%）
    match checkpoint.stage.as_str() {
        "language" => super::mark_stage_complete("language"),
        "class" => {
            super::mark_stage_complete("language");
            super::mark_stage_complete("class");
        }
        "repair" => {
            super::mark_stage_complete("language");
            super::mark_stage_complete("class");
            super::mark_stage_complete("repair");
        }
        _ => {}
    }
    let mut work_graph = checkpoint
        .work_graph
        .as_ref()
        .and_then(|value| serde_json::from_value::<WorkGraphSnapshot>(value.clone()).ok())
        .map(WorkGraph::from_snapshot)
        .unwrap_or_else(|| WorkGraph::new(task_id));
    let mut class_ledger = ClassDecisionLedger {
        decisions: checkpoint
            .class_exclusions
            .iter()
            .map(|id| {
                (
                    id.clone(),
                    ClassDecision {
                        action: "exclude".to_string(),
                        translation: None,
                        reason: Some("断点续传时已排除".to_string()),
                    },
                )
            })
            .collect(),
        replaced_files: checkpoint.class_changed_files.clone(),
        replacement_count: checkpoint.class_replacement_count,
    };
    let mut completed_batches = checkpoint.completed_language_batches.clone();

    // 翻译记忆：全局共享（工作区根目录），任务结束统一 flush
    let memory_root = workspace
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let mut memory = TranslationMemory::load(memory::memory_path(&memory_root));

    let lang_progress = {
        let app = app.clone();
        move |progress: f64, message: &str, retry: Option<super::RetryInfo>| {
            update_status(&app, "language", progress, message, retry)
        }
    };
    let class_progress = {
        let app = app.clone();
        move |progress: f64, message: &str, retry: Option<super::RetryInfo>| {
            update_status(&app, "class", progress, message, retry)
        }
    };

    let mut cancelled = false;
    let mut fatal: Option<String> = None;

    // 1) 语言翻译（跳过已完成批次）
    if has_language {
        let mut pending = inspection.clone();
        pending
            .language_sources
            .retain(|s| s.required_count() > 0 && !completed_batches.contains(&s.target_path));
        if !pending.language_sources.is_empty() {
            update_status(&app, "language", 10.0, "开始翻译", None);
            match translate_lang::run_language_route(
                &workspace,
                &pending,
                &config,
                &model,
                batch_size,
                &mut memory,
                &mut work_graph,
                &cancel_flag,
                &lang_progress,
            )
            .await
            {
                Ok(()) => completed_batches.extend(
                    pending
                        .language_sources
                        .iter()
                        .map(|s| s.target_path.clone()),
                ),
                Err(e) if e == CANCEL_MSG || cancel_flag.load(Ordering::Relaxed) => {
                    cancelled = true
                }
                Err(e) => {
                    log_error!("[ModTranslation] 语言翻译失败: {e}");
                    fatal = Some(e)
                }
            }
            if !cancelled && fatal.is_none() {
                checkpoint.completed_language_batches = completed_batches.clone();
                checkpoint.work_graph =
                    Some(serde_json::to_value(work_graph.snapshot()).unwrap_or_default());
                checkpoint.stage = "language".to_string();
                let _ = resume::save_checkpoint(&workspace, &checkpoint);
            }
        }
    }

    // 2) class 常量池文本翻译（确定性排除 → AI 判定 → 改写写回）
    if !cancelled
        && fatal.is_none()
        && class_text_enabled
        && !inspection.class_candidates.is_empty()
    {
        update_status(&app, "class", 90.0, "class 文本判定", None);
        match translate_class::run_class_route(
            &workspace,
            &inspection,
            &config,
            &model,
            &mut class_ledger,
            &cancel_flag,
            &class_progress,
        )
        .await
        {
            Ok(()) => {}
            Err(e) if e == CANCEL_MSG || cancel_flag.load(Ordering::Relaxed) => cancelled = true,
            Err(e) => {
                log_error!("[ModTranslation] class 文本翻译失败: {e}");
                fatal = Some(e)
            }
        }
        checkpoint.class_exclusions = class_ledger.snapshot_exclusions();
        checkpoint.class_changed_files = class_ledger.replaced_files.clone();
        checkpoint.class_replacement_count = class_ledger.replacement_count;
        checkpoint.work_graph =
            Some(serde_json::to_value(work_graph.snapshot()).unwrap_or_default());
        checkpoint.stage = "class".to_string();
        let _ = resume::save_checkpoint(&workspace, &checkpoint);
    }

    // 3) 质量回修兜底（复验 → AI 修复 → 原子写回）
    if !cancelled && fatal.is_none() && repair_enabled {
        let sources: Vec<&LanguageSource> = inspection
            .language_sources
            .iter()
            .filter(|s| s.required_count() > 0)
            .collect();
        if !sources.is_empty() {
            update_status(&app, "repair", 0.0, "质量复验中", None);
            let repair_progress = {
                let app = app.clone();
                move |progress: f64, message: &str, retry: Option<super::RetryInfo>| {
                    update_status(&app, "repair", progress, message, retry)
                }
            };
            match repair::run_repair_passes(
                &workspace,
                &sources,
                &mut work_graph,
                &config,
                &model,
                &cancel_flag,
                &repair_progress,
            )
            .await
            {
                Ok(true) => {}
                Ok(false) => log_warn!("[ModTranslation] 质量回修存在残留问题，继续打包"),
                Err(e) if e == CANCEL_MSG || cancel_flag.load(Ordering::Relaxed) => {
                    cancelled = true
                }
                Err(e) => {
                    log_error!("[ModTranslation] 质量回修失败: {e}");
                    fatal = Some(e)
                }
            }
            if !cancelled && fatal.is_none() {
                checkpoint.work_graph =
                    Some(serde_json::to_value(work_graph.snapshot()).unwrap_or_default());
                checkpoint.stage = "repair".to_string();
                let _ = resume::save_checkpoint(&workspace, &checkpoint);
            }
        }
    }

    // 4) 结果处理：取消/失败保留工作区供续传；成功打包后清理
    let outcome = if cancelled {
        Some(TaskSnapshot {
            status: "cancelled".to_string(),
            stage: "translate".to_string(),
            progress: 0.0,
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

    if let Some(snapshot) = outcome {
        let done = snapshot.status == "completed";
        finish(&app, snapshot);
        if done {
            cleanup(&workspace);
        } else {
            log_info!("[ModTranslation] 任务未完成，保留工作区供断点续传");
        }
    } else {
        cleanup(&workspace);
    }
    if let Err(e) = memory.flush() {
        log_warn!("[ModTranslation] 保存翻译记忆失败: {e}");
    }
    super::clear_running();
}

/// 解包到缓存工作区并分析；优先复用可续传工作区，否则新建 `job-<hash>` 并写续传标记
pub(super) fn prepare(jar_path: &Path) -> Result<Prepared, String> {
    let hash = analyze::file_hash(jar_path)?;
    let root = Cache::instance()
        .ensure_dir(WORKSPACE_ROOT)
        .map_err(|e| format!("无法创建模组翻译缓存目录: {e}"))?;

    // 断点续传：命中同 hash + 身份的工作区则复用（跳过已翻译批次）
    if let Some(workspace) = resume::find_resumable_workspace(&root, &hash, RESUME_IDENTITY) {
        if let Some(marker) = resume::read_resume_marker(&workspace) {
            log_info!("[ModTranslation] 发现可续传工作区，跳过已翻译批次");
            return Ok(Prepared {
                checkpoint: resume::read_checkpoint(&workspace),
                workspace,
                inspection: marker.inspection,
            });
        }
    }

    let workspace = root.join(format!("job-{}", &hash[..16]));
    if workspace.exists() {
        std::fs::remove_dir_all(&workspace).map_err(|e| format!("无法清理旧工作区: {e}"))?;
    }
    std::fs::create_dir_all(&workspace).map_err(|e| format!("无法创建工作区: {e}"))?;

    let extracted = jar::extract_archive(jar_path, &workspace, &jar::ExtractionLimits::default())?;
    if extracted.signed {
        log_warn!("[ModTranslation] JAR 含签名文件，重打包后签名将失效");
    }
    let inspection = analyze::inspect_jar(&workspace, jar_path, extracted.signed);
    log_info!(
        "[ModTranslation] 分析完成：{}（{} 个语言源，{} 条目）",
        inspection.original_filename,
        inspection.language_sources.len(),
        inspection.language_entries
    );
    let _ = resume::write_resume_marker(
        &workspace,
        &resume::ResumeMarker {
            version: 1,
            input_hash: hash,
            resume_identity: RESUME_IDENTITY.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            inspection: inspection.clone(),
        },
    );
    Ok(Prepared {
        checkpoint: None,
        workspace,
        inspection,
    })
}

/// 清理工作区（任务结束后释放缓存空间）
fn cleanup(workspace: &Path) {
    if let Err(e) = std::fs::remove_dir_all(workspace) {
        log_warn!("[ModTranslation] 清理工作区失败: {e}");
    }
}
