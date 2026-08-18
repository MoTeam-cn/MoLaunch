//! class 翻译路由单元测试（经 translate_class.rs 的 #[path] 子模块引入）

use super::*;
use crate::mod_translation::types::{Loader, Quote};

fn sample_inspection(candidates: Vec<ClassCandidate>) -> JarInspection {
    JarInspection {
        input_path: std::path::PathBuf::from("demo.jar"),
        original_filename: "demo.jar".to_string(),
        loader: Loader::Fabric,
        mod_ids: vec!["demo".to_string()],
        project_names: vec!["Demo".to_string()],
        version: None,
        signed: false,
        language_sources: Vec::new(),
        language_entries: 0,
        class_candidates: candidates,
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

fn candidate(id: &str, path: &str, text: &str) -> ClassCandidate {
    ClassCandidate {
        id: id.to_string(),
        path: path.to_string(),
        paths: vec![path.to_string()],
        occurrences: 1,
        text: text.to_string(),
    }
}

#[test]
fn deterministic_exclusions_are_recorded_without_model_calls() {
    let candidates = vec![
        candidate("c1", "a/b/SomeClass.class", "com.example.SomeClass"),
        candidate(
            "c2",
            "xaero/map/gui/GuiMap.class",
            "Failed to load your map. Retry?",
        ),
    ];
    let inspection = sample_inspection(candidates);
    let mut ledger = ClassDecisionLedger::default();
    resolve_deterministic_exclusions(&inspection, &mut ledger);
    assert_eq!(ledger.decisions["c1"].action, "exclude");
    assert_eq!(
        ledger.decisions["c1"].reason.as_deref(),
        Some("java_class_name")
    );
    // GUI 宿主路径保留：不入账本，进入 AI 判定
    assert!(!ledger.decisions.contains_key("c2"));
    assert_eq!(ledger.unresolved(&inspection.class_candidates).len(), 1);
}

#[test]
fn class_decisions_validation_rejects_incomplete_and_invalid() {
    let candidates = vec![candidate("c1", "demo/A.class", "Iron Ingot")];
    // translate 缺中文译文 → 拒绝
    let bad = format!(
        r#"{{"decisions":[{{"id":"c1","action":"translate","translation":"Iron Ingot","reason":"visible"}}]}}"#
    );
    assert!(parse_and_validate_decisions(&bad, &candidates).is_err());
    // 缺少 decisions 数组 → 拒绝
    assert!(parse_and_validate_decisions(r#"{"foo":1}"#, &candidates).is_err());
    // 未覆盖全部候选 → 拒绝
    let partial =
        format!(r#"{{"decisions":[{{"id":"c1","action":"exclude","reason":"internal"}}]}}"#);
    let two = vec![
        candidate("c1", "demo/A.class", "Iron Ingot"),
        candidate("c2", "demo/B.class", "Hello"),
    ];
    assert!(parse_and_validate_decisions(&partial, &two).is_err());
}

#[test]
fn class_decisions_validation_accepts_valid_translate() {
    let candidates = vec![candidate("c1", "demo/A.class", "Spawn %d zombies")];
    let ok = r#"{"decisions":[{"id":"c1","action":"translate","translation":"生成 %d 只僵尸","reason":"visible"}]}"#;
    let decisions = parse_and_validate_decisions(ok, &candidates).unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].translation.as_deref(), Some("生成 %d 只僵尸"));
    // 占位符不一致 → 拒绝
    let mismatch = r#"{"decisions":[{"id":"c1","action":"translate","translation":"生成 %s 只僵尸","reason":"visible"}]}"#;
    assert!(parse_and_validate_decisions(mismatch, &candidates).is_err());
}
