//! 断点续传测试

use std::path::PathBuf;

use super::{
    find_resumable_workspace, read_checkpoint, save_checkpoint, write_resume_marker, Checkpoint,
    ResumeMarker,
};
use crate::mod_translation::types::{JarInspection, Loader, Quote};

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
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!("molaunch-resume-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn marker(input_hash: &str, created_at: &str) -> ResumeMarker {
    ResumeMarker {
        version: 1,
        input_hash: input_hash.to_string(),
        resume_identity: "molaunch-mod-translation-v1".to_string(),
        created_at: created_at.to_string(),
        inspection: sample_inspection(),
    }
}

#[test]
fn test_find_resumable_workspace_matches_hash_and_identity() {
    let root = temp_dir("match");
    let job = root.join("job-demo");
    std::fs::create_dir_all(&job).expect("mkdir job");
    write_resume_marker(&job, &marker("abc123", "2026-08-18T00:00:00Z")).expect("write marker");

    assert_eq!(
        find_resumable_workspace(&root, "abc123", "molaunch-mod-translation-v1"),
        Some(job.clone())
    );
    assert_eq!(
        find_resumable_workspace(&root, "other", "molaunch-mod-translation-v1"),
        None
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_newest_matching_workspace_is_selected() {
    let root = temp_dir("newest");
    for (name, created_at) in [
        ("job-old", "2026-08-17T00:00:00Z"),
        ("job-new", "2026-08-18T00:00:00Z"),
    ] {
        let job = root.join(name);
        std::fs::create_dir_all(&job).expect("mkdir job");
        write_resume_marker(&job, &marker("abc123", created_at)).expect("write marker");
    }
    assert_eq!(
        find_resumable_workspace(&root, "abc123", "molaunch-mod-translation-v1"),
        Some(root.join("job-new"))
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_checkpoint_round_trips_and_filters_version() {
    let dir = temp_dir("checkpoint");
    let mut checkpoint = Checkpoint::fresh("task-1".to_string());
    checkpoint.completed_language_batches = vec!["assets/x/lang/zh_cn.json".to_string()];
    checkpoint.class_replacement_count = 3;
    checkpoint.stage = "language".to_string();
    checkpoint.work_graph = Some(serde_json::json!({"done": true}));
    save_checkpoint(&dir, &checkpoint).expect("save checkpoint");

    let restored = read_checkpoint(&dir).expect("checkpoint should be readable");
    assert_eq!(restored.version, 1);
    assert_eq!(restored.task_id, "task-1");
    assert_eq!(restored.completed_language_batches.len(), 1);
    assert_eq!(restored.stage, "language");
    assert_eq!(restored.work_graph, Some(serde_json::json!({"done": true})));

    let mut future = checkpoint.clone();
    future.version = 99;
    save_checkpoint(&dir, &future).expect("save future checkpoint");
    assert!(read_checkpoint(&dir).is_none());

    let _ = std::fs::remove_dir_all(&dir);
}
