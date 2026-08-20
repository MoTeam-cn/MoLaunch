//! 模组翻译：重打包（生成 `<原名>-zh_cn.jar` 与完成报告）

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::log_info;

use super::jar;
use super::ledger::{ClassDecisionLedger, WorkGraph, WorkKind, WorkStatus};
use super::status::{failed_snapshot, update_status};
use super::types::{JarInspection, ModNameResult, TaskSnapshot, TranslationReport};

/// 重打包为 `<原名>-zh_cn.jar`，成功时附带完成报告（mod_name / 工作图统计 / class 统计）
pub(super) fn package(
    app: &AppHandle,
    workspace: &Path,
    jar_path: &Path,
    inspection: &JarInspection,
    mod_name: Option<ModNameResult>,
    work_graph: &WorkGraph,
    class_ledger: &ClassDecisionLedger,
) -> Option<TaskSnapshot> {
    update_status(app, "package", 95.0, "正在打包", None);
    let manifest = match jar::ArchiveManifest::read(workspace) {
        Some(m) => m,
        None => return Some(failed_snapshot("package", "缺少归档清单，无法重打包")),
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
            let language_attempted = work_graph
                .items
                .values()
                .filter(|i| i.kind == WorkKind::Language)
                .count();
            let language_accepted = work_graph
                .items
                .values()
                .filter(|i| i.kind == WorkKind::Language && i.status == WorkStatus::Verified)
                .count();
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
                mod_name: mod_name.clone(),
                report: Some(TranslationReport {
                    task_id: String::new(),
                    ok: true,
                    output_path: output_path.to_string_lossy().to_string(),
                    mod_name,
                    language_attempted,
                    language_accepted,
                    class_resolved: class_ledger.decisions.len(),
                    class_total: inspection.class_candidates.len(),
                    warnings: inspection.warnings.clone(),
                }),
                ..TaskSnapshot::new(String::new())
            })
        }
        Err(e) => Some(failed_snapshot("package", &e)),
    }
}

/// 输出路径：同目录 `<原名>-zh_cn.jar`
pub(super) fn output_path_for(jar_path: &Path) -> PathBuf {
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
