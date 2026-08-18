//! 质量回修：修复方案应用与原子写回

use std::collections::BTreeMap;
use std::path::Path;

use crate::mod_translation::lang;
use crate::mod_translation::ledger::{WorkGraph, WorkKind};
use crate::mod_translation::repair::{issue_id, RepairAction};
use crate::mod_translation::types::{LanguageKind, LanguageSource};

pub(crate) fn apply_actions(
    workspace: &Path,
    source: &LanguageSource,
    actions: &[RepairAction],
    work_graph: &mut WorkGraph,
) -> Result<(), String> {
    let mut translations = BTreeMap::new();
    for action in actions {
        let key = action_key(source, &action.issue_id)?;
        match action.action.as_str() {
            "translate" => {
                let translation = action.translation.as_deref().unwrap_or_default();
                translations.insert(key, translation.to_string());
            }
            "keep-source" => {
                let text = source.entries.get(&key).cloned().unwrap_or_default();
                translations.insert(key, text);
            }
            other => return Err(format!("未知 action 类型：{other}")),
        }
    }
    if !translations.is_empty() {
        write_target(workspace, source, &translations)?;
    }
    for action in actions {
        let key = action_key(source, &action.issue_id)?;
        let item_id = work_graph.upsert(
            WorkKind::Language,
            "翻译".to_string(),
            format!("{}#{}", source.target_path, key),
            1.0,
        );
        if action.action == "keep-source" {
            work_graph.supersede(&item_id, "显式保留原文");
        }
        work_graph.record_attempt(&item_id, "repair".to_string(), "ok".to_string(), None);
    }
    Ok(())
}

fn action_key(source: &LanguageSource, want_id: &str) -> Result<String, String> {
    for key in source.entries.keys() {
        if issue_id("language", &format!("{}#{}", source.target_path, key)) == want_id {
            return Ok(key.clone());
        }
    }
    Err(format!("未知 issue：{want_id}"))
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
            let mut snap = lang::snapshot_free_text(&original);
            snap.target_lines = lang::align_free_text(&snap.source_lines, translations);
            lang::render_localized_text(&snap)
        }
    };
    let tmp = target_path.with_extension("tmp");
    std::fs::write(&tmp, output).map_err(|e| format!("无法写入 {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &target_path)
        .map_err(|e| format!("无法替换 {}: {e}", target_path.display()))
}
