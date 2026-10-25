//! class 处置账本：候选处置记录与排除快照

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::mod_translation::types::ClassCandidate;

/// 处置结论：排除（不进输出 JAR）
pub const VERDICT_EXCLUDE: &str = "exclude";
/// 处置结论：改写为中文
pub const VERDICT_TRANSLATE: &str = "translate";

/// 单个候选的处置记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDecision {
    pub verdict: String,
    pub text: Option<String>,
    pub note: Option<String>,
}

impl ClassDecision {
    fn excluded(&self) -> bool {
        self.verdict == VERDICT_EXCLUDE
    }
}

/// class 处置账本：候选 id → 处置记录
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassDecisionLedger {
    records: BTreeMap<String, ClassDecision>,
    /// 已改写的文件（相对路径，去重）
    pub touched_files: Vec<String>,
    /// 累计改写出现次数
    pub rewritten_occurrences: usize,
}

impl ClassDecisionLedger {
    pub fn from_map(records: BTreeMap<String, ClassDecision>) -> Self {
        Self {
            records,
            ..Default::default()
        }
    }

    pub fn to_map(&self) -> BTreeMap<String, ClassDecision> {
        self.records.clone()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn decision(&self, id: &str) -> Option<&ClassDecision> {
        self.records.get(id)
    }

    /// 记录一次处置
    pub fn record(&mut self, id: &str, verdict: &str, text: Option<&str>, note: Option<&str>) {
        self.records.insert(
            id.to_string(),
            ClassDecision {
                verdict: verdict.to_string(),
                text: text.map(str::to_string),
                note: note.map(str::to_string),
            },
        );
    }

    /// 尚未处置的候选（保持输入顺序）
    pub fn pending<'a>(&self, candidates: &'a [ClassCandidate]) -> Vec<&'a ClassCandidate> {
        candidates
            .iter()
            .filter(|candidate| !self.records.contains_key(&candidate.id))
            .collect()
    }

    /// 被排除的候选 id（打包时跳过）
    pub fn excluded_ids(&self) -> Vec<String> {
        self.records
            .iter()
            .filter(|(_, decision)| decision.excluded())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 记一次文件改写（去重）
    pub fn mark_touched(&mut self, relative: &str) {
        if !self.touched_files.iter().any(|path| path == relative) {
            self.touched_files.push(relative.to_string());
        }
    }
}
