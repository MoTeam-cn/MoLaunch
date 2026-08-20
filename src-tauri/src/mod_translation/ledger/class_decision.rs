//! class 处置账本：候选决策记录与排除快照

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::mod_translation::types::ClassCandidate;

/// class 候选处置决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDecision {
    pub action: String,
    pub translation: Option<String>,
    pub reason: Option<String>,
}

/// class 处置账本
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassDecisionLedger {
    pub decisions: HashMap<String, ClassDecision>,
    pub replaced_files: Vec<String>,
    pub replacement_count: usize,
}

impl ClassDecisionLedger {
    /// 尚未处置的候选
    pub fn unresolved(&self, candidates: &[ClassCandidate]) -> Vec<ClassCandidate> {
        candidates
            .iter()
            .filter(|candidate| !self.decisions.contains_key(&candidate.id))
            .cloned()
            .collect()
    }

    /// 被排除的候选 id（打包时跳过）
    pub fn snapshot_exclusions(&self) -> Vec<String> {
        self.decisions
            .iter()
            .filter(|(_, decision)| decision.action == "exclude")
            .map(|(id, _)| id.clone())
            .collect()
    }
}
