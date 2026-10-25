//! 模组翻译：重打包（生成 `<原名>-zh_cn.jar` 与完成报告）

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::log_info;

use super::jar;
use super::ledger::{ClassDecisionLedger, WorkGraph, WorkKind, WorkStatus};
use super::status::{failed_snapshot, update_status};
use super::types::{JarInspection, ModNameResult, TaskSnapshot, TranslationReport};

/// 重打包为 `<原名>-zh_cn.jar`，成功时附带完成报告
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

    if jar::WorkspaceMeta::read(workspace).is_none() {
        return Some(failed_snapshot("package", "缺少工作区元数据，无法重打包"));
    }
    let output_path = output_path_for(jar_path);
    if output_path.exists() {
        return Some(failed_snapshot(
            "package",
            &format!("输出文件已存在: {}", output_path.display()),
        ));
    }

    if let Err(error) = jar::package_archive(workspace, &output_path) {
        return Some(failed_snapshot("package", &error));
    }

    log_info!(
        "[ModTranslation] 完成：{}（{} 条目）",
        output_path.display(),
        inspection.language_entries
    );
    Some(completed_snapshot(
        &output_path,
        inspection,
        mod_name,
        work_graph,
        class_ledger,
    ))
}

/// 语言条目统计：(尝试数, 通过数)
fn language_counts(work_graph: &WorkGraph) -> (usize, usize) {
    let language_items = work_graph
        .items
        .values()
        .filter(|item| item.kind == WorkKind::Language);
    let attempted = language_items.clone().count();
    let accepted = language_items
        .filter(|item| item.status == WorkStatus::Verified)
        .count();
    (attempted, accepted)
}

fn completed_snapshot(
    output_path: &Path,
    inspection: &JarInspection,
    mod_name: Option<ModNameResult>,
    work_graph: &WorkGraph,
    class_ledger: &ClassDecisionLedger,
) -> TaskSnapshot {
    let (language_attempted, language_accepted) = language_counts(work_graph);
    let output = output_path.to_string_lossy().to_string();

    TaskSnapshot {
        status: "completed".to_string(),
        stage: "package".to_string(),
        progress: 100.0,
        message: "翻译完成".to_string(),
        output_path: Some(output.clone()),
        mod_name: mod_name.clone(),
        report: Some(TranslationReport {
            task_id: String::new(),
            ok: true,
            output_path: output,
            mod_name,
            language_attempted,
            language_accepted,
            class_resolved: class_ledger.len(),
            class_total: inspection.class_candidates.len(),
            warnings: inspection.warnings.clone(),
        }),
        ..TaskSnapshot::new(String::new())
    }
}

/// 输出路径：同目录 `<原名>-zh_cn.jar`
pub(super) fn output_path_for(jar_path: &Path) -> PathBuf {
    let name = jar_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();
    let (stem, extension) = match name.rsplit_once('.') {
        Some((stem, extension)) => (stem.to_string(), format!(".{extension}")),
        None => (name, String::new()),
    };
    let filename = format!("{stem}-zh_cn{extension}");
    match jar_path.parent() {
        Some(directory) => directory.join(filename),
        None => PathBuf::from(filename),
    }
}
