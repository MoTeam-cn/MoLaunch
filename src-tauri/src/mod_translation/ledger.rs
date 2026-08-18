//! 模组翻译：任务状态中枢（工作图、任务记忆、class 处置账本）

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::mod_translation::types::ClassCandidate;

/// 工作项类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkKind {
    Language,
    VisibleText,
}

/// 工作项状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkStatus {
    Pending,
    Claimed,
    Submitted,
    Verified,
    Superseded,
}

/// 一次处理尝试
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub action: String,
    pub outcome: String,
    #[serde(default)]
    pub failure_class: Option<String>,
}

/// 单个工作项（按 source 去重）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub kind: WorkKind,
    pub goal: String,
    pub source: String,
    pub weight: f64,
    #[serde(default)]
    pub attempts: Vec<Attempt>,
    pub status: WorkStatus,
    pub version: u64,
}

/// 工作图快照（序列化/恢复用）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkGraphSnapshot {
    pub task_id: String,
    pub revision: u64,
    pub items: Vec<WorkItem>,
}

/// 任务工作图：全部工作项的状态中枢
#[derive(Debug, Clone)]
pub struct WorkGraph {
    pub task_id: String,
    pub items: BTreeMap<String, WorkItem>,
    pub revision: u64,
}

impl WorkGraph {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            items: BTreeMap::new(),
            revision: 0,
        }
    }

    /// 新增或更新工作项（按 source 去重），返回稳定 id
    pub fn upsert(&mut self, kind: WorkKind, goal: String, source: String, weight: f64) -> String {
        let id = work_item_id(&self.task_id, kind, &source);
        match self.items.get_mut(&id) {
            Some(item) => {
                item.goal = goal;
                item.weight = weight;
            }
            None => {
                self.items.insert(
                    id.clone(),
                    WorkItem {
                        id: id.clone(),
                        kind,
                        goal,
                        source,
                        weight,
                        attempts: Vec::new(),
                        status: WorkStatus::Pending,
                        version: 1,
                    },
                );
                self.revision += 1;
            }
        }
        id
    }

    /// 记录一次处理尝试
    pub fn record_attempt(
        &mut self,
        id: &str,
        action: String,
        outcome: String,
        failure_class: Option<String>,
    ) {
        if let Some(item) = self.items.get_mut(id) {
            item.attempts.push(Attempt {
                action,
                outcome,
                failure_class,
            });
            self.revision += 1;
        }
    }

    /// 验收：accepted → Verified；否则回 Pending（Superseded 不可逆）
    pub fn reconcile(&mut self, id: &str, accepted: bool, reason: &str) {
        if let Some(item) = self.items.get_mut(id) {
            if accepted {
                item.status = WorkStatus::Verified;
            } else if item.status != WorkStatus::Superseded {
                item.status = WorkStatus::Pending;
            }
            item.version += 1;
            self.revision += 1;
            let _ = reason;
        }
    }

    /// 重试：清空模型类尝试（fast/deep 翻译与质量回修），保留 quality_audit，置回 Pending
    pub fn reset_for_retry(&mut self, id: &str) {
        if let Some(item) = self.items.get_mut(id) {
            if item.status != WorkStatus::Superseded {
                item.status = WorkStatus::Pending;
            }
            item.attempts.retain(|attempt| {
                !matches!(
                    attempt.action.as_str(),
                    "fast_translate" | "deep_translate" | "deep_quality" | "agent"
                )
            });
            item.version += 1;
            self.revision += 1;
        }
    }

    /// 置为已替代（终态，不可逆）
    pub fn supersede(&mut self, id: &str, reason: &str) {
        if let Some(item) = self.items.get_mut(id) {
            item.status = WorkStatus::Superseded;
            item.version += 1;
            self.revision += 1;
            let _ = reason;
        }
    }

    /// 进度：(verified 权重, 总权重)
    pub fn progress(&self) -> (f64, f64) {
        let total = self.items.values().map(|item| item.weight).sum::<f64>();
        let verified = self
            .items
            .values()
            .filter(|item| item.status == WorkStatus::Verified)
            .map(|item| item.weight)
            .sum::<f64>();
        (verified, total)
    }

    /// 模型类尝试次数（fast_translate/deep_translate/deep_quality/agent）
    pub fn model_attempt_count(&self, id: &str) -> usize {
        self.items
            .get(id)
            .map(|item| {
                item.attempts
                    .iter()
                    .filter(|attempt| {
                        matches!(
                            attempt.action.as_str(),
                            "fast_translate" | "deep_translate" | "deep_quality" | "agent"
                        )
                    })
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn snapshot(&self) -> WorkGraphSnapshot {
        WorkGraphSnapshot {
            task_id: self.task_id.clone(),
            revision: self.revision,
            items: self.items.values().cloned().collect(),
        }
    }

    pub fn from_snapshot(s: WorkGraphSnapshot) -> Self {
        Self {
            task_id: s.task_id,
            items: s
                .items
                .into_iter()
                .map(|item| (item.id.clone(), item))
                .collect(),
            revision: s.revision,
        }
    }
}

/// 工作项 id：SHA256(task_id + kind + source) 前 24 位 hex
fn work_item_id(task_id: &str, kind: WorkKind, source: &str) -> String {
    let kind_str = match kind {
        WorkKind::Language => "language",
        WorkKind::VisibleText => "visible_text",
    };
    let mut hasher = Sha256::new();
    hasher.update(task_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(kind_str.as_bytes());
    hasher.update(b"\0");
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())[..24].to_string()
}

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
            self.decisions.extend(decisions.iter().take(50).cloned());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_dedupes_by_source_with_stable_id() {
        let mut graph = WorkGraph::new("task-1".to_string());
        let id_a = graph.upsert(
            WorkKind::Language,
            "翻译".to_string(),
            "assets/demo/lang/en_us.json#key".to_string(),
            1.0,
        );
        let id_b = graph.upsert(
            WorkKind::Language,
            "翻译（更新）".to_string(),
            "assets/demo/lang/en_us.json#key".to_string(),
            2.0,
        );
        assert_eq!(id_a, id_b);
        assert_eq!(id_a.len(), 24);
        assert_eq!(graph.items.len(), 1);
        assert_eq!(graph.items[&id_a].weight, 2.0);
        assert_eq!(graph.items[&id_a].version, 1);
    }

    #[test]
    fn reset_for_retry_clears_model_attempts_keeps_quality_audit() {
        let mut graph = WorkGraph::new("task-2".to_string());
        let id = graph.upsert(
            WorkKind::Language,
            "翻译".to_string(),
            "src".to_string(),
            1.0,
        );
        graph.record_attempt(
            &id,
            "fast_translate".to_string(),
            "bad".to_string(),
            Some("partial".to_string()),
        );
        graph.record_attempt(
            &id,
            "quality_audit".to_string(),
            "bad".to_string(),
            Some("quality".to_string()),
        );
        graph.reconcile(&id, true, "ok");

        graph.reset_for_retry(&id);

        let item = &graph.items[&id];
        assert_eq!(item.status, WorkStatus::Pending);
        assert_eq!(graph.model_attempt_count(&id), 0);
        assert_eq!(item.attempts.len(), 1);
        assert_eq!(item.attempts[0].action, "quality_audit");
    }

    #[test]
    fn progress_sums_verified_weight() {
        let mut graph = WorkGraph::new("task-3".to_string());
        let id_a = graph.upsert(
            WorkKind::Language,
            "a".to_string(),
            "src-a".to_string(),
            1.0,
        );
        let id_b = graph.upsert(
            WorkKind::VisibleText,
            "b".to_string(),
            "src-b".to_string(),
            3.0,
        );
        graph.reconcile(&id_a, true, "ok");
        graph.supersede(&id_b, "merged");
        let (verified, total) = graph.progress();
        assert_eq!(verified, 1.0);
        assert_eq!(total, 4.0);
    }

    #[test]
    fn supersede_is_irreversible() {
        let mut graph = WorkGraph::new("task-4".to_string());
        let id = graph.upsert(WorkKind::Language, "a".to_string(), "src".to_string(), 1.0);
        graph.supersede(&id, "replaced");
        graph.reconcile(&id, false, "retry");
        assert_eq!(graph.items[&id].status, WorkStatus::Superseded);
    }

    #[test]
    fn task_memory_update_is_bounded() {
        let mut memory = TaskMemory::default();
        let glossary = (0..600)
            .map(|i| serde_json::json!({"source": format!("s{i}"), "translation": format!("t{i}")}))
            .collect::<Vec<_>>();
        let decisions = (0..300)
            .map(|i| serde_json::json!({"i": i}))
            .collect::<Vec<_>>();
        memory.update(serde_json::json!({
            "recommendedName": "x".repeat(100),
            "summary": "y".repeat(5000),
            "glossary": glossary,
            "decisions": decisions,
            "uncertainties": (0..200).map(|i| format!("u{i}")).collect::<Vec<_>>(),
            "discoveredTargets": (0..600).map(|i| format!("t{i}")).collect::<Vec<_>>(),
        }));
        assert_eq!(
            memory.recommended_name.as_ref().unwrap().chars().count(),
            80
        );
        assert_eq!(memory.summary.chars().count(), 4000);
        assert_eq!(memory.glossary.len(), 500);
        assert_eq!(memory.decisions.len(), 200);
        assert_eq!(memory.uncertainties.len(), 100);
        assert_eq!(memory.discovered_targets.len(), 500);
    }
}
