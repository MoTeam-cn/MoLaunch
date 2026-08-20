//! class 处置应用：改写写回与账本记录

use std::path::Path;

use super::super::class;
use super::super::ledger::{ClassDecision, ClassDecisionLedger};
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
        .find(|c| c.id == decision.id)
        .ok_or_else(|| format!("未知候选：{}", decision.id))?;
    match decision.action.as_str() {
        "exclude" => {
            record_decision(
                ledger,
                &candidate.id,
                "exclude",
                None,
                decision.reason.as_deref(),
            );
            Ok(())
        }
        "translate" => {
            let translation = decision
                .translation
                .clone()
                .ok_or_else(|| format!("class 候选 {} 缺少译文", candidate.id))?;
            apply_class_translation(workspace, candidate, &translation, ledger)
        }
        other => Err(format!("未知动作：{other}")),
    }
}

fn apply_class_translation(
    workspace: &Path,
    candidate: &ClassCandidate,
    translation: &str,
    class_ledger: &mut ClassDecisionLedger,
) -> Result<(), String> {
    if candidate.text == translation {
        record_decision(
            class_ledger,
            &candidate.id,
            "exclude",
            None,
            Some("译文与原文一致"),
        );
        return Ok(());
    }
    let paths = match candidate.paths.as_slice() {
        [] => vec![candidate.path.clone()],
        _ => candidate.paths.clone(),
    };
    for relative in &paths {
        let target_path = workspace.join(relative);
        let bytes = std::fs::read(&target_path)
            .map_err(|e| format!("无法读取 {}: {e}", target_path.display()))?;
        let rewritten = class::replace_class_utf8(&bytes, &candidate.text, translation)?;
        if rewritten == bytes {
            return Err(format!("class 候选在声明路径中不存在：{relative}"));
        }
        let temporary = target_path.with_extension("class.tmp");
        std::fs::write(&temporary, &rewritten)
            .map_err(|e| format!("无法写入 {}: {e}", temporary.display()))?;
        std::fs::rename(&temporary, &target_path)
            .map_err(|e| format!("无法移动 {}: {e}", target_path.display()))?;
        if !class_ledger.replaced_files.contains(relative) {
            class_ledger.replaced_files.push(relative.clone());
        }
    }
    record_decision(
        class_ledger,
        &candidate.id,
        "translate",
        Some(translation),
        None,
    );
    class_ledger.replacement_count += paths.len();
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
