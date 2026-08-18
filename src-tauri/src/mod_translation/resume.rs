//! 断点续传：工作区匹配 + 检查点读写（临时文件 + rename 原子写）。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mod_translation::types::JarInspection;

pub const RESUME_FILE: &str = ".mod-translator-resume.json";
pub const CHECKPOINT_FILE: &str = ".mod-translator-checkpoint.json";
/// 当前检查点版本（读取时过滤不兼容版本）
const CHECKPOINT_VERSION: u32 = 1;

/// 续传标记：记录输入哈希与身份，用于跨会话匹配工作区
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeMarker {
    pub version: u32,
    pub input_hash: String,
    pub resume_identity: String,
    pub created_at: String,
    pub inspection: JarInspection,
}

/// 检查点：任务进度快照（语言批次 / class 处理 / 工作图）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkpoint {
    pub version: u32,
    pub task_id: String,
    pub completed_language_batches: Vec<String>,
    pub class_exclusions: Vec<String>,
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

/// 扫描工作区根目录下 `job-*` 目录，返回匹配 input_hash + identity 的最新工作区
pub fn find_resumable_workspace(
    workspace_root: &Path,
    input_hash: &str,
    identity: &str,
) -> Option<PathBuf> {
    let entries = std::fs::read_dir(workspace_root).ok()?;
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || !entry.file_name().to_string_lossy().starts_with("job-") {
            continue;
        }
        let Some(marker) = read_resume_marker(&path) else {
            continue;
        };
        if marker.input_hash == input_hash && marker.resume_identity == identity {
            matches.push((marker.created_at, path));
        }
    }
    matches.sort_by(|a, b| b.0.cmp(&a.0));
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
        .filter(|c| (1..=CHECKPOINT_VERSION).contains(&c.version))
}

pub fn save_checkpoint(directory: &Path, checkpoint: &Checkpoint) -> Result<(), String> {
    let content =
        serde_json::to_string(checkpoint).map_err(|e| format!("序列化检查点失败: {e}"))?;
    let path = directory.join(CHECKPOINT_FILE);
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, content).map_err(|e| format!("写入检查点失败: {e}"))?;
    std::fs::rename(&temporary, &path).map_err(|e| format!("落盘检查点失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mod_translation::types::{Loader, Quote};

    fn sample_inspection() -> JarInspection {
        JarInspection {
            input_path: PathBuf::from("C:/mods/demo.jar"),
            original_filename: "demo.jar".to_string(),
            loader: Loader::Fabric,
            mod_ids: vec!["demo".to_string()],
            project_names: vec!["Demo".to_string()],
            version: Some("1.0.0".to_string()),
            signed: false,
            language_sources: Vec::new(),
            language_entries: 0,
            class_candidates: Vec::new(),
            coverage: Vec::new(),
            quote: Quote {
                estimated_input_tokens: 0,
                estimated_output_tokens: 0,
                estimated_tokens: 0,
                estimated_calls: 0,
                language_batches: 0,
                class_batches: 0,
                points: 0,
                characters: 0,
                entries: 0,
            },
            mod_name: None,
            existing_chinese: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "molaunch-resume-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn marker(input_hash: &str, created_at: &str) -> ResumeMarker {
        ResumeMarker {
            version: 1,
            input_hash: input_hash.to_string(),
            resume_identity: "mod-translator-v1".to_string(),
            created_at: created_at.to_string(),
            inspection: sample_inspection(),
        }
    }

    #[test]
    fn find_resumable_workspace_matches_hash_and_identity() {
        let root = temp_dir("match");
        let job = root.join("job-demo");
        std::fs::create_dir_all(&job).unwrap();
        write_resume_marker(&job, &marker("abc123", "2026-08-18T00:00:00Z")).unwrap();
        assert_eq!(
            find_resumable_workspace(&root, "abc123", "mod-translator-v1"),
            Some(job.clone())
        );
        assert_eq!(
            find_resumable_workspace(&root, "other", "mod-translator-v1"),
            None
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn newest_matching_workspace_is_selected() {
        let root = temp_dir("newest");
        for (name, created_at) in [
            ("job-old", "2026-08-17T00:00:00Z"),
            ("job-new", "2026-08-18T00:00:00Z"),
        ] {
            let job = root.join(name);
            std::fs::create_dir_all(&job).unwrap();
            write_resume_marker(&job, &marker("abc123", created_at)).unwrap();
        }
        assert_eq!(
            find_resumable_workspace(&root, "abc123", "mod-translator-v1"),
            Some(root.join("job-new"))
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn checkpoint_round_trips_and_filters_version() {
        let dir = temp_dir("checkpoint");
        let mut checkpoint = Checkpoint::fresh("task-1".to_string());
        checkpoint.completed_language_batches = vec!["assets/x/lang/zh_cn.json".to_string()];
        checkpoint.class_replacement_count = 3;
        checkpoint.stage = "language".to_string();
        checkpoint.work_graph = Some(serde_json::json!({"done": true}));
        save_checkpoint(&dir, &checkpoint).unwrap();
        let restored = read_checkpoint(&dir).expect("检查点应可恢复");
        assert_eq!(restored.version, 1);
        assert_eq!(restored.task_id, "task-1");
        assert_eq!(restored.completed_language_batches.len(), 1);
        assert_eq!(restored.stage, "language");
        assert_eq!(restored.work_graph, Some(serde_json::json!({"done": true})));
        // 不兼容版本被过滤
        let mut future = checkpoint.clone();
        future.version = 99;
        save_checkpoint(&dir, &future).unwrap();
        assert!(read_checkpoint(&dir).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
