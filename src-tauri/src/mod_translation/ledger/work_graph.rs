//! 任务工作图：全部工作项的状态中枢（规格 §5 断点续传的状态载体）
//!
//! 工作项按「任务 id + 类型 + 源」生成稳定 id，状态机只有五态：
//! Pending → Claimed → Submitted → Verified，另加不可逆的 Superseded。
//! 每次状态变化都推进 `version`，图整体推进 `revision`，供续传判断是否需要落盘。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 模型类尝试：重试时清空，质量审计等非模型尝试保留
const MODEL_ACTIONS: [&str; 4] = ["fast_translate", "deep_translate", "deep_quality", "agent"];

fn is_model_action(action: &str) -> bool {
    MODEL_ACTIONS.contains(&action)
}

/// 工作项类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkKind {
    Language,
    VisibleText,
}

impl WorkKind {
    fn label(self) -> &'static str {
        match self {
            WorkKind::Language => "language",
            WorkKind::VisibleText => "visible_text",
        }
    }
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

/// 工作图快照（序列化 / 恢复用）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkGraphSnapshot {
    pub task_id: String,
    pub revision: u64,
    pub items: Vec<WorkItem>,
}

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
        if let Some(item) = self.items.get_mut(&id) {
            item.goal = goal;
            item.weight = weight;
            return id;
        }
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
        id
    }

    /// 记录一次处理尝试（不改变状态）
    pub fn record_attempt(
        &mut self,
        id: &str,
        action: String,
        outcome: String,
        failure_class: Option<String>,
    ) {
        let Some(item) = self.items.get_mut(id) else {
            return;
        };
        item.attempts.push(Attempt {
            action,
            outcome,
            failure_class,
        });
        self.revision += 1;
    }

    /// 验收：通过则 Verified，否则回到 Pending（Superseded 不回退）
    pub fn reconcile(&mut self, id: &str, accepted: bool, reason: &str) {
        let _ = reason;
        let Some(item) = self.items.get_mut(id) else {
            return;
        };
        if accepted {
            item.status = WorkStatus::Verified;
        } else if item.status != WorkStatus::Superseded {
            item.status = WorkStatus::Pending;
        }
        item.version += 1;
        self.revision += 1;
    }

    /// 重试：清空模型类尝试，保留质量审计记录，状态回到 Pending
    pub fn reset_for_retry(&mut self, id: &str) {
        let Some(item) = self.items.get_mut(id) else {
            return;
        };
        if item.status != WorkStatus::Superseded {
            item.status = WorkStatus::Pending;
        }
        item.attempts
            .retain(|attempt| !is_model_action(&attempt.action));
        item.version += 1;
        self.revision += 1;
    }

    /// 置为已替代（终态）
    pub fn supersede(&mut self, id: &str, reason: &str) {
        let _ = reason;
        let Some(item) = self.items.get_mut(id) else {
            return;
        };
        item.status = WorkStatus::Superseded;
        item.version += 1;
        self.revision += 1;
    }

    /// 进度：(已验证权重, 总权重)
    pub fn progress(&self) -> (f64, f64) {
        let total: f64 = self.items.values().map(|item| item.weight).sum();
        let verified: f64 = self
            .items
            .values()
            .filter(|item| item.status == WorkStatus::Verified)
            .map(|item| item.weight)
            .sum();
        (verified, total)
    }

    /// 已发生的模型类尝试次数（重试上限判断用）
    pub fn model_attempt_count(&self, id: &str) -> usize {
        self.items
            .get(id)
            .map(|item| {
                item.attempts
                    .iter()
                    .filter(|attempt| is_model_action(&attempt.action))
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

    pub fn from_snapshot(snapshot: WorkGraphSnapshot) -> Self {
        Self {
            task_id: snapshot.task_id,
            items: snapshot
                .items
                .into_iter()
                .map(|item| (item.id.clone(), item))
                .collect(),
            revision: snapshot.revision,
        }
    }
}

/// 工作项 id：sha256(task_id ␀ kind ␀ source) 前 24 位
fn work_item_id(task_id: &str, kind: WorkKind, source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(task_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(kind.label().as_bytes());
    hasher.update(b"\0");
    hasher.update(source.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    digest[..24].to_string()
}
