//! 任务工作图：全部工作项的状态中枢

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
