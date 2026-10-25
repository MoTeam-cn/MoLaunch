//! class 处置应用：改写写回与账本记录

use std::path::Path;

use super::super::class;
use super::super::ledger::{ClassDecision, ClassDecisionLedger};
use super::super::types::ClassCandidate;
use super::prompt::DecisionEntry;

const EXCLUDE_ACTION: &str = "exclude";
const TRANSLATE_ACTION: &str = "translate";

pub(crate) fn apply_decision(
    workspace: &Path,
    candidates: &[&ClassCandidate],
    decision: &DecisionEntry,
    ledger: &mut ClassDecisionLedger,
) -> Result<(), String> {
    let candidate = candidates
        .iter()
        .copied()
        .find(|candidate| candidate.id == decision.id)
        .ok_or_else(|| format!("未知候选：{}", decision.id))?;

    match decision.action.as_str() {
        EXCLUDE_ACTION => {
            record_decision(
                ledger,
                &candidate.id,
                EXCLUDE_ACTION,
                None,
                decision.reason.as_deref(),
            );
            Ok(())
        }
        TRANSLATE_ACTION => {
            let translation = decision
                .translation
                .clone()
                .ok_or_else(|| format!("class 候选 {} 缺少译文", candidate.id))?;
            apply_translation(workspace, candidate, &translation, ledger)
        }
        other => Err(format!("未知动作：{other}")),
    }
}

fn apply_translation(
    workspace: &Path,
    candidate: &ClassCandidate,
    translation: &str,
    ledger: &mut ClassDecisionLedger,
) -> Result<(), String> {
    if candidate.text == translation {
        record_decision(
            ledger,
            &candidate.id,
            EXCLUDE_ACTION,
            None,
            Some("译文与原文一致"),
        );
        return Ok(());
    }

    let paths: Vec<String> = if candidate.paths.is_empty() {
        vec![candidate.path.clone()]
    } else {
        candidate.paths.clone()
    };

    for relative in &paths {
        let target_path = workspace.join(relative);
        let bytes = std::fs::read(&target_path)
            .map_err(|e| format!("无法读取 {}: {e}", target_path.display()))?;
        let rewritten = class::replace_class_utf8(&bytes, &candidate.text, translation)?;
        if rewritten == bytes {
            return Err(format!("class 候选在声明路径中不存在：{relative}"));
        }
        crate::utils::fs::write_atomic(&target_path, &rewritten)?;
        if !ledger.replaced_files.contains(relative) {
            ledger.replaced_files.push(relative.clone());
        }
    }

    record_decision(
        ledger,
        &candidate.id,
        TRANSLATE_ACTION,
        Some(translation),
        None,
    );
    ledger.replacement_count += paths.len();
    Ok(())
}

pub(crate) fn record_decision(
    ledger: &mut ClassDecisionLedger,
    id: &str,
    action: &str,
    translation: Option<&str>,
    reason: Option<&str>,
) {
    ledger.decisions.insert(
        id.to_string(),
        ClassDecision {
            action: action.to_string(),
            translation: translation.map(str::to_string),
            reason: reason.map(str::to_string),
        },
    );
}
