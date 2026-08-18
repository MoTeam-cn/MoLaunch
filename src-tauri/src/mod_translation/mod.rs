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
pub mod prompt;
pub mod quality;
pub mod resume;
pub mod translate;
pub mod types;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter};

use crate::ai_core;
use crate::storage::cache::Cache;
use crate::{log_info, log_warn};

use self::translate::{translate_sources, CANCEL_MSG};
use self::types::{
    AnalyzeParams, AnalyzeResult, JarInspection, SourceSummary, StartParams, TaskSnapshot,
};

/// 进度事件名（前端经 useTauriEvent 订阅）
pub const EVENT_NAME: &str = "mod-translation-event";
/// 缓存工作区根目录（`.Molaunch/cache/mod-translation`）
const WORKSPACE_ROOT: &str = "mod-translation";

/// 已就绪的分析结果（analyze 与 start 之间传递工作区）
#[derive(Clone)]
struct Prepared {
    workspace: PathBuf,
    inspection: JarInspection,
}

/// 进行中的任务（cancel_flag 置位后翻译循环尽快中止）
struct RunningTask {
    cancel_flag: Arc<AtomicBool>,
}

static PREPARED: Mutex<Option<Prepared>> = Mutex::new(None);
static RUNNING: Mutex<Option<RunningTask>> = Mutex::new(None);
static STATUS: Mutex<Option<TaskSnapshot>> = Mutex::new(None);

/// 分析 JAR：解包 → 探测加载器 → 汇总语言源
pub async fn analyze_jar(params: AnalyzeParams) -> Result<AnalyzeResult, String> {
    ensure_idle()?;
    let jar_path = PathBuf::from(params.jar_path);
    if !jar_path.is_file() {
        return Err(format!("文件不存在: {}", jar_path.display()));
    }
    let prepared = prepare(&jar_path)?;
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
        _ => prepare(&jar_path)?,
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
        run_task(
            app_for_task,
            prepared,
            jar_path,
            config,
            model,
            params.batch_size as usize,
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
            message: String::new(),
            output_path: None,
            error: None,
            mod_name: None,
            report: None,
        })
}

/// 后台任务主体：翻译 → 打包 → 清理工作区
async fn run_task(
    app: AppHandle,
    prepared: Prepared,
    jar_path: PathBuf,
    config: ai_core::AiConfig,
    model: String,
    batch_size: usize,
    cancel_flag: Arc<AtomicBool>,
) {
    let Prepared {
        workspace,
        inspection,
    } = prepared;

    let total_required: usize = inspection
        .language_sources
        .iter()
        .map(|s| s.required_count())
        .sum();
    if total_required == 0 {
        finish(
            &app,
            TaskSnapshot {
                status: "failed".to_string(),
                stage: "translate".to_string(),
                progress: 0.0,
                message: "没有需要翻译的条目".to_string(),
                error: Some("未找到可翻译的 en_us 文本，或内容已全部翻译".to_string()),
                ..TaskSnapshot::new(String::new())
            },
        );
        cleanup(&workspace);
        clear_running();
        return;
    }

    update_status(&app, "translate", 0.0, "开始翻译");
    let app_for_progress = app.clone();
    let progress = move |progress: f64, message: &str| {
        update_status(&app_for_progress, "translate", progress, message);
    };
    let result = translate_sources(
        &workspace,
        &inspection,
        &config,
        &model,
        batch_size,
        &cancel_flag,
        &progress,
    )
    .await;

    let outcome = match result {
        Ok(()) => {
            if cancel_flag.load(Ordering::Relaxed) {
                Some(TaskSnapshot {
                    status: "cancelled".to_string(),
                    stage: "translate".to_string(),
                    progress: 0.0,
                    message: "任务已取消".to_string(),
                    ..TaskSnapshot::new(String::new())
                })
            } else {
                package(&app, &workspace, &jar_path, &inspection)
            }
        }
        Err(e) if e == CANCEL_MSG || cancel_flag.load(Ordering::Relaxed) => Some(TaskSnapshot {
            status: "cancelled".to_string(),
            stage: "translate".to_string(),
            progress: 0.0,
            message: "任务已取消".to_string(),
            ..TaskSnapshot::new(String::new())
        }),
        Err(e) => Some(TaskSnapshot {
            status: "failed".to_string(),
            stage: "translate".to_string(),
            progress: 0.0,
            message: "翻译失败".to_string(),
            error: Some(e),
            ..TaskSnapshot::new(String::new())
        }),
    };

    if let Some(snapshot) = outcome {
        finish(&app, snapshot);
    }
    cleanup(&workspace);
    clear_running();
}

/// 重打包为 `<原名>-zh_cn.jar`，返回完成快照
fn package(
    app: &AppHandle,
    workspace: &Path,
    jar_path: &Path,
    inspection: &JarInspection,
) -> Option<TaskSnapshot> {
    update_status(app, "package", 95.0, "正在打包");
    let manifest = match jar::ArchiveManifest::read(workspace) {
        Some(m) => m,
        None => {
            return Some(failed_snapshot("package", "缺少归档清单，无法重打包"));
        }
    };
    let output_path = output_path_for(jar_path);
    if output_path.exists() {
        return Some(failed_snapshot(
            "package",
            &format!("输出文件已存在: {}", output_path.display()),
        ));
    }
    match jar::package_archive(workspace, &output_path, &manifest) {
        Ok(()) => {
            log_info!(
                "[ModTranslation] 完成：{}（{} 条目）",
                output_path.display(),
                inspection.language_entries
            );
            Some(TaskSnapshot {
                status: "completed".to_string(),
                stage: "package".to_string(),
                progress: 100.0,
                message: "翻译完成".to_string(),
                output_path: Some(output_path.to_string_lossy().to_string()),
                ..TaskSnapshot::new(String::new())
            })
        }
        Err(e) => Some(failed_snapshot("package", &e)),
    }
}

/// 输出路径：同目录 `<原名>-zh_cn.jar`
fn output_path_for(jar_path: &Path) -> PathBuf {
    let name = jar_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) => (s.to_string(), format!(".{e}")),
        None => (name, String::new()),
    };
    let filename = format!("{stem}-zh_cn{ext}");
    match jar_path.parent() {
        Some(dir) => dir.join(filename),
        None => PathBuf::from(filename),
    }
}

/// 解包到缓存工作区并分析（工作区按 JAR 哈希命名，每次重建）
fn prepare(jar_path: &Path) -> Result<Prepared, String> {
    let hash = analyze::file_hash(jar_path)?;
    let root = Cache::instance()
        .ensure_dir(WORKSPACE_ROOT)
        .map_err(|e| format!("无法创建模组翻译缓存目录: {e}"))?;
    let workspace = root.join(hash.chars().take(16).collect::<String>());
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
    Ok(Prepared {
        workspace,
        inspection,
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
        warnings: inspection.warnings.clone(),
    }
}

fn failed_snapshot(stage: &str, error: &str) -> TaskSnapshot {
    TaskSnapshot {
        status: "failed".to_string(),
        stage: stage.to_string(),
        progress: 0.0,
        message: "任务失败".to_string(),
        error: Some(error.to_string()),
        ..TaskSnapshot::new(String::new())
    }
}

/// 更新状态并向前端 emit 进度事件
fn update_status(app: &AppHandle, stage: &str, progress: f64, message: &str) {
    let mut snapshot = current_status();
    snapshot.stage = stage.to_string();
    snapshot.progress = progress;
    snapshot.message = message.to_string();
    store_status(&snapshot);
    let _ = app.emit(EVENT_NAME, &snapshot);
}

/// 终态：写入状态 + emit 事件（task_id 为空时从当前状态补全）
fn finish(app: &AppHandle, mut snapshot: TaskSnapshot) {
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

fn clear_running() {
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

/// 清理工作区（任务结束后释放缓存空间）
fn cleanup(workspace: &Path) {
    if let Err(e) = std::fs::remove_dir_all(workspace) {
        log_warn!("[ModTranslation] 清理工作区失败: {e}");
    }
}
