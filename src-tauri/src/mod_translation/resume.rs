//! 断点续传：工作区匹配 + 检查点读写（原子写）

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mod_translation::ledger::ClassDecision;
use crate::mod_translation::types::JarInspection;

pub const RESUME_FILE: &str = ".molaunch-translation-resume.json";
pub const CHECKPOINT_FILE: &str = ".molaunch-translation-checkpoint.json";
/// 当前检查点版本（读取时过滤不兼容版本）
const CHECKPOINT_VERSION: u32 = 1;
/// 工作区目录前缀
const WORKSPACE_PREFIX: &str = "job-";

/// 续传标记：输入哈希 + 身份，用于跨会话匹配工作区
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeMarker {
    pub version: u32,
    pub input_hash: String,
    pub resume_identity: String,
    pub created_at: String,
    pub inspection: JarInspection,
}

/// 检查点：任务进度快照（语言批次 / class 处置 / 工作图）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkpoint {
    pub version: u32,
    pub task_id: String,
    pub completed_language_batches: Vec<String>,
    pub class_exclusions: Vec<String>,
    /// 完整 class 处置决策（含译文），续传时恢复避免重复判定
    #[serde(default)]
    pub class_decisions: BTreeMap<String, ClassDecision>,
    pub class_replacement_count: usize,
    pub class_changed_files: Vec<String>,
    #[serde(default)]
    pub work_graph: Option<serde_json::Value>,
    #[serde(default)]
    pub stage: String,
}

impl Checkpoint {
    pub fn fresh(task_id: String) -> Self {
        Self {
            version: CHECKPOINT_VERSION,
            task_id,
            ..Default::default()
        }
    }
}

/// 扫描 `job-*` 目录，返回哈希 + 身份都匹配的最新工作区
pub fn find_resumable_workspace(
    workspace_root: &Path,
    input_hash: &str,
    identity: &str,
) -> Option<PathBuf> {
    let entries = std::fs::read_dir(workspace_root).ok()?;
    let mut matches: Vec<(String, PathBuf)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir()
            || !entry
                .file_name()
                .to_string_lossy()
                .starts_with(WORKSPACE_PREFIX)
        {
            continue;
        }
        let Some(marker) = read_resume_marker(&path) else {
            continue;
        };
        if marker.input_hash == input_hash && marker.resume_identity == identity {
            matches.push((marker.created_at, path));
        }
    }

    matches.sort_by(|left, right| right.0.cmp(&left.0));
    matches.into_iter().next().map(|(_, path)| path)
}

pub fn read_resume_marker(directory: &Path) -> Option<ResumeMarker> {
    let content = std::fs::read_to_string(directory.join(RESUME_FILE)).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_resume_marker(directory: &Path, marker: &ResumeMarker) -> Result<(), String> {
    let content = serde_json::to_string(marker).map_err(|e| format!("序列化续传标记失败: {e}"))?;
    std::fs::write(directory.join(RESUME_FILE), content)
        .map_err(|e| format!("写入续传标记失败: {e}"))
}

pub fn read_checkpoint(directory: &Path) -> Option<Checkpoint> {
    let content = std::fs::read_to_string(directory.join(CHECKPOINT_FILE)).ok()?;
    serde_json::from_str::<Checkpoint>(&content)
        .ok()
        .filter(|checkpoint| (1..=CHECKPOINT_VERSION).contains(&checkpoint.version))
}

pub fn save_checkpoint(directory: &Path, checkpoint: &Checkpoint) -> Result<(), String> {
    let content =
        serde_json::to_string(checkpoint).map_err(|e| format!("序列化检查点失败: {e}"))?;
    crate::utils::fs::write_atomic(&directory.join(CHECKPOINT_FILE), content.as_bytes())
}

#[cfg(test)]
#[path = "resume_test.rs"]
mod tests;
