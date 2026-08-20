//! 任务记忆：跨会话继承，各字段有界

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 任务记忆：跨会话继承，各字段有界
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

impl TaskMemory {
    /// 合并一次增量更新（camelCase 字段），超限截断
    pub fn update(&mut self, update: serde_json::Value) {
        if let Some(name) = update
            .get("recommendedName")
            .and_then(serde_json::Value::as_str)
        {
            self.recommended_name = Some(name.chars().take(80).collect());
        }
        if let Some(summary) = update.get("summary").and_then(serde_json::Value::as_str) {
            self.summary = summary.chars().take(4000).collect();
        }
        if let Some(glossary) = update.get("glossary").and_then(serde_json::Value::as_array) {
            for entry in glossary {
                if let (Some(source), Some(translation)) = (
                    entry.get("source").and_then(serde_json::Value::as_str),
                    entry.get("translation").and_then(serde_json::Value::as_str),
                ) {
                    self.glossary.insert(
                        source.chars().take(120).collect(),
                        translation.chars().take(120).collect(),
                    );
                }
            }
            while self.glossary.len() > 500 {
                if let Some(key) = self.glossary.keys().next().cloned() {
                    self.glossary.remove(&key);
                }
            }
        }
        if let Some(decisions) = update
            .get("decisions")
            .and_then(serde_json::Value::as_array)
        {
            self.decisions.extend(decisions.iter().cloned());
            while self.decisions.len() > 200 {
                self.decisions.remove(0);
            }
        }
        if let Some(uncertainties) = update
            .get("uncertainties")
            .and_then(serde_json::Value::as_array)
        {
            for value in uncertainties.iter().filter_map(serde_json::Value::as_str) {
                self.uncertainties.push(value.chars().take(500).collect());
            }
            while self.uncertainties.len() > 100 {
                self.uncertainties.remove(0);
            }
        }
        if let Some(targets) = update
            .get("discoveredTargets")
            .and_then(serde_json::Value::as_array)
        {
            for value in targets.iter().filter_map(serde_json::Value::as_str) {
                self.discovered_targets
                    .push(value.chars().take(500).collect());
            }
            while self.discovered_targets.len() > 500 {
                self.discovered_targets.remove(0);
            }
        }
    }
}
