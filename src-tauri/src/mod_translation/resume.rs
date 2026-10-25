//! 断点续传：工作区匹配 + 检查点读写（原子写）
//!
//! 检查点按阶段分组：语言进度与 class 进度各自成段，任务级信息放外层。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mod_translation::ledger::ClassDecision;
use crate::mod_translation::types::JarInspection;

/// 续传标记文件名
pub const MARKER_FILE: &str = ".molaunch-resume-marker.json";
/// 检查点文件名
pub const STATE_FILE: &str = ".molaunch-resume-state.json";
/// 工作区目录前缀
const WORKSPACE_PREFIX: &str = "job-";
/// 当前检查点版本（读取时过滤不兼容版本）
const STATE_VERSION: u32 = 1;

/// 续传标记：输入哈希 + 身份，用于跨会话匹配工作区
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeMarker {
    pub version: u32,
    pub input_hash: String,
    pub resume_identity: String,
    pub created_at: String,
    pub inspection: JarInspection,
}

/// 语言阶段进度
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageProgress {
    /// 已完成的语言源目标路径
    pub finished: Vec<String>,
}

/// class 阶段进度
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassProgress {
    /// 已处置的候选（含译文，续传时直接复用）
    #[serde(default)]
    pub decisions: BTreeMap<String, ClassDecision>,
    /// 被排除的候选 id
    pub excluded: Vec<String>,
    /// 已改写的文件
    pub replaced: Vec<String>,
    /// 累计改写出现次数
    pub occurrences: usize,
}

/// 检查点：任务进度快照
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Checkpoint {
    pub version: u32,
    pub task_id: String,
    /// 已完成阶段：language / class / repair
    pub stage: String,
    pub language: LanguageProgress,
    pub class: ClassProgress,
    #[serde(default)]
    pub graph: Option<serde_json::Value>,
}

impl Checkpoint {
    pub fn fresh(task_id: String) -> Self {
        Self {
            version: STATE_VERSION,
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
    let mut newest: Option<(String, PathBuf)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if !path.is_dir() || !name.to_string_lossy().starts_with(WORKSPACE_PREFIX) {
            continue;
        }
        let Some(marker) = read_resume_marker(&path) else {
            continue;
        };
        if marker.input_hash != input_hash || marker.resume_identity != identity {
            continue;
        }
        if newest
            .as_ref()
            .is_none_or(|(created_at, _)| marker.created_at > *created_at)
        {
            newest = Some((marker.created_at, path));
        }
    }

    newest.map(|(_, path)| path)
}

pub fn read_resume_marker(directory: &Path) -> Option<ResumeMarker> {
    let content = std::fs::read_to_string(directory.join(MARKER_FILE)).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_resume_marker(directory: &Path, marker: &ResumeMarker) -> Result<(), String> {
    let payload = serde_json::to_string(marker).map_err(|e| format!("序列化续传标记失败: {e}"))?;
    std::fs::write(directory.join(MARKER_FILE), payload)
        .map_err(|e| format!("写入续传标记失败: {e}"))
}

pub fn read_checkpoint(directory: &Path) -> Option<Checkpoint> {
    let content = std::fs::read_to_string(directory.join(STATE_FILE)).ok()?;
    serde_json::from_str::<Checkpoint>(&content)
        .ok()
        .filter(|state| (1..=STATE_VERSION).contains(&state.version))
}

pub fn save_checkpoint(directory: &Path, checkpoint: &Checkpoint) -> Result<(), String> {
    let payload =
        serde_json::to_string(checkpoint).map_err(|e| format!("序列化检查点失败: {e}"))?;
    crate::utils::fs::write_atomic(&directory.join(STATE_FILE), payload.as_bytes())
}

#[cfg(test)]
#[path = "resume_test.rs"]
mod tests;
