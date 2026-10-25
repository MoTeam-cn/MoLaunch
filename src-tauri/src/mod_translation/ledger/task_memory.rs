//! 任务记忆：跨会话继承的术语表与决策记录，各字段有界

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const NAME_LIMIT: usize = 80;
const SUMMARY_LIMIT: usize = 4000;
const TERM_LIMIT: usize = 120;
const GLOSSARY_LIMIT: usize = 500;
const DECISION_LIMIT: usize = 200;
const NOTE_LIMIT: usize = 500;
const UNCERTAINTY_LIMIT: usize = 100;
const TARGET_LIMIT: usize = 500;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskMemory {
    #[serde(default)]
    pub recommended_name: Option<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub glossary: HashMap<String, String>,
    #[serde(default)]
    pub decisions: Vec<serde_json::Value>,
    #[serde(default)]
    pub uncertainties: Vec<String>,
    #[serde(default)]
    pub discovered_targets: Vec<String>,
}

/// 截断到指定字符数
fn clip(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

/// 只保留最后 `limit` 项
fn keep_last<T>(items: &mut Vec<T>, limit: usize) {
    if items.len() > limit {
        let excess = items.len() - limit;
        items.drain(0..excess);
    }
}

/// 从数组里取字符串字段
fn strings_of(value: &serde_json::Value, key: &str) -> Option<Vec<String>> {
    let array = value.get(key)?.as_array()?;
    Some(
        array
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(|item| clip(item, NOTE_LIMIT))
            .collect(),
    )
}

impl TaskMemory {
    /// 合并一次增量更新（字段名 camelCase），超限截断
    pub fn update(&mut self, update: serde_json::Value) {
        if let Some(name) = update
            .get("recommendedName")
            .and_then(serde_json::Value::as_str)
        {
            self.recommended_name = Some(clip(name, NAME_LIMIT));
        }
        if let Some(summary) = update.get("summary").and_then(serde_json::Value::as_str) {
            self.summary = clip(summary, SUMMARY_LIMIT);
        }
        if let Some(entries) = update.get("glossary").and_then(serde_json::Value::as_array) {
            for entry in entries {
                let source = entry.get("source").and_then(serde_json::Value::as_str);
                let translation = entry.get("translation").and_then(serde_json::Value::as_str);
                if let (Some(source), Some(translation)) = (source, translation) {
                    self.glossary
                        .insert(clip(source, TERM_LIMIT), clip(translation, TERM_LIMIT));
                }
            }
            while self.glossary.len() > GLOSSARY_LIMIT {
                let Some(key) = self.glossary.keys().next().cloned() else {
                    break;
                };
                self.glossary.remove(&key);
            }
        }
        if let Some(decisions) = update
            .get("decisions")
            .and_then(serde_json::Value::as_array)
        {
            self.decisions.extend(decisions.iter().cloned());
            keep_last(&mut self.decisions, DECISION_LIMIT);
        }
        if let Some(items) = strings_of(&update, "uncertainties") {
            self.uncertainties.extend(items);
            keep_last(&mut self.uncertainties, UNCERTAINTY_LIMIT);
        }
        if let Some(items) = strings_of(&update, "discoveredTargets") {
            self.discovered_targets.extend(items);
            keep_last(&mut self.discovered_targets, TARGET_LIMIT);
        }
    }
}
