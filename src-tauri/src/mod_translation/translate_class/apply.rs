//! class 处置应用：改写写回与账本记录

use std::path::Path;

use super::super::class;
use super::super::ledger::{ClassDecisionLedger, VERDICT_EXCLUDE, VERDICT_TRANSLATE};
use super::super::types::ClassCandidate;
use super::prompt::DecisionEntry;

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
        VERDICT_EXCLUDE => {
            record_decision(
                ledger,
                &candidate.id,
                VERDICT_EXCLUDE,
                None,
                decision.reason.as_deref(),
            );
            Ok(())
        }
        VERDICT_TRANSLATE => {
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
            VERDICT_EXCLUDE,
            None,
            Some("译文与原文一致"),
        );
        return Ok(());
    }

    let targets: Vec<&String> = if candidate.paths.is_empty() {
        vec![&candidate.path]
    } else {
        candidate.paths.iter().collect()
    };

    for relative in &targets {
        let path = workspace.join(relative);
        let original =
            std::fs::read(&path).map_err(|e| format!("无法读取 {}: {e}", path.display()))?;
        let rewritten = class::replace_class_utf8(&original, &candidate.text, translation)?;
        if rewritten == original {
            return Err(format!("class 候选在声明路径中不存在：{relative}"));
        }
        crate::utils::fs::write_atomic(&path, &rewritten)?;
        ledger.mark_touched(relative);
    }

    record_decision(
        ledger,
        &candidate.id,
        VERDICT_TRANSLATE,
        Some(translation),
        None,
    );
    ledger.rewritten_occurrences += targets.len();
    Ok(())
}

pub(crate) fn record_decision(
    ledger: &mut ClassDecisionLedger,
    id: &str,
    verdict: &str,
    text: Option<&str>,
    note: Option<&str>,
) {
    ledger.record(id, verdict, text, note);
}
