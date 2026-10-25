//! 质量回修：修复方案应用与原子写回

use std::collections::BTreeMap;
use std::path::Path;

use crate::mod_translation::lang;
use crate::mod_translation::ledger::{WorkGraph, WorkKind};
use crate::mod_translation::repair::{issue_id, RepairAction};
use crate::mod_translation::types::{LanguageKind, LanguageSource};

const TRANSLATE_ACTION: &str = "translate";
const KEEP_SOURCE_ACTION: &str = "keep-source";

/// 应用一批修复方案：先汇总译文，再一次性写回，最后记账
pub(crate) fn apply_actions(
    workspace: &Path,
    source: &LanguageSource,
    actions: &[RepairAction],
    work_graph: &mut WorkGraph,
) -> Result<(), String> {
    let resolved = resolve_actions(source, actions)?;

    let mut translations = BTreeMap::new();
    for (action, key) in &resolved {
        let text = match action.action.as_str() {
            TRANSLATE_ACTION => action.translation.clone().unwrap_or_default(),
            KEEP_SOURCE_ACTION => source.entries.get(key).cloned().unwrap_or_default(),
            other => return Err(format!("未知 action 类型：{other}")),
        };
        translations.insert(key.clone(), text);
    }

    if !translations.is_empty() {
        write_target(workspace, source, &translations)?;
    }

    for (action, key) in resolved {
        let item_id = work_graph.upsert(
            WorkKind::Language,
            "翻译".to_string(),
            format!("{}#{}", source.target_path, key),
            1.0,
        );
        if action.action == KEEP_SOURCE_ACTION {
            work_graph.supersede(&item_id, "显式保留原文");
        }
        work_graph.record_attempt(&item_id, "repair".to_string(), "ok".to_string(), None);
    }
    Ok(())
}

/// issue id → (action, 语言条目键)
fn resolve_actions(
    source: &LanguageSource,
    actions: &[RepairAction],
) -> Result<Vec<(RepairAction, String)>, String> {
    actions
        .iter()
        .map(|action| {
            let key = action_key(source, &action.issue_id)?;
            Ok((action.clone(), key))
        })
        .collect()
}

fn action_key(source: &LanguageSource, wanted: &str) -> Result<String, String> {
    source
        .entries
        .keys()
        .find(|key| issue_id("language", &format!("{}#{}", source.target_path, key)) == wanted)
        .cloned()
        .ok_or_else(|| format!("未知 issue：{wanted}"))
}

fn write_target(
    workspace: &Path,
    source: &LanguageSource,
    translations: &BTreeMap<String, String>,
) -> Result<(), String> {
    let source_path = workspace.join(&source.source_path);
    let target_path = workspace.join(&source.target_path);
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("无法创建目标目录: {e}"))?;
    }
    let original = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("无法读取 {}: {e}", source_path.display()))?;

    let output = match source.kind {
        LanguageKind::Json => lang::write_json_lang(&original, translations)?,
        LanguageKind::KeyValue => {
            lang::write_keyvalue(&lang::parse_keyvalue(&original).1, translations)
        }
        LanguageKind::StructuredJson => lang::apply_structured_strings(&original, translations)?,
        LanguageKind::FreeText => {
            let mut snapshot = lang::snapshot_free_text(&original);
            snapshot.target_lines = lang::align_free_text(&snapshot.source_lines, translations);
            lang::render_localized_text(&snapshot)
        }
    };
    crate::utils::fs::write_atomic(&target_path, output.as_bytes())
}
