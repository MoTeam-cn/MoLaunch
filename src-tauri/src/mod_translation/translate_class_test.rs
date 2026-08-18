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
fn class_decisions_validation_rejects_invalid_json() {
    let candidates = vec![candidate("c1", "demo/A.class", "Iron Ingot")];
    let refs: Vec<&ClassCandidate> = candidates.iter().collect();
    // 缺少 decisions 数组 → 拒绝
    assert!(parse_and_validate_decisions(r#"{"foo":1}"#, &refs).is_err());
    // 非 JSON → 拒绝
    assert!(parse_and_validate_decisions("not json", &refs).is_err());
}

#[test]
fn class_decisions_validation_drops_invalid_and_reports_uncovered() {
    let candidates = vec![
        candidate("c1", "demo/A.class", "Iron Ingot"),
        candidate("c2", "demo/B.class", "Hello"),
    ];
    let refs: Vec<&ClassCandidate> = candidates.iter().collect();
    // translate 缺中文译文 → 丢弃该 decision，返回未覆盖
    let bad = format!(
        r#"{{"decisions":[{{"id":"c1","action":"translate","translation":"Iron Ingot","reason":"visible"}}]}}"#
    );
    let (decisions, uncovered) = parse_and_validate_decisions(&bad, &refs).unwrap();
    assert!(decisions.is_empty());
    assert_eq!(uncovered.len(), 2);
    // 只覆盖 c1 → 宽容返回未覆盖 c2
    let partial =
        format!(r#"{{"decisions":[{{"id":"c1","action":"exclude","reason":"internal"}}]}}"#);
    let (decisions, uncovered) = parse_and_validate_decisions(&partial, &refs).unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].id, "c1");
    assert_eq!(uncovered.len(), 1);
    assert_eq!(uncovered[0].id, "c2");
    // 未知 id → 丢弃
    let unknown = r#"{"decisions":[{"id":"nope","action":"exclude","reason":"internal"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(unknown, &refs).unwrap();
    assert!(decisions.is_empty());
    assert_eq!(uncovered.len(), 2);
    // 重复 id → 丢弃后者
    let dup = r#"{"decisions":[{"id":"c1","action":"exclude","reason":"a"},{"id":"c1","action":"exclude","reason":"b"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(dup, &refs).unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(uncovered.len(), 1);
    assert_eq!(uncovered[0].id, "c2");
}

#[test]
fn class_decisions_validation_accepts_valid_translate() {
    let candidates = vec![candidate("c1", "demo/A.class", "Spawn %d zombies")];
    let refs: Vec<&ClassCandidate> = candidates.iter().collect();
    let ok = r#"{"decisions":[{"id":"c1","action":"translate","translation":"生成 %d 只僵尸","reason":"visible"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(ok, &refs).unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].translation.as_deref(), Some("生成 %d 只僵尸"));
    assert!(uncovered.is_empty());
    // 占位符不一致 → 丢弃
    let mismatch = r#"{"decisions":[{"id":"c1","action":"translate","translation":"生成 %s 只僵尸","reason":"visible"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(mismatch, &refs).unwrap();
    assert!(decisions.is_empty());
    assert_eq!(uncovered.len(), 1);
}

#[test]
fn class_decisions_validation_tolerates_edited_id() {
    let candidates = vec![
        candidate("037980176fd0aea1aebd7ab2", "demo/A.class", "Iron Ingot"),
        candidate("12b1fb25cf755a9ce87c22a4", "demo/B.class", "Hello"),
    ];
    let refs: Vec<&ClassCandidate> = candidates.iter().collect();
    // 模型改写 id（删除一个字符）→ 容错匹配到原候选
    let edited = r#"{"decisions":[{"id":"037980176fd0aea1ebd7ab2","action":"exclude","reason":"internal"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(edited, &refs).unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].id, "037980176fd0aea1aebd7ab2");
    assert_eq!(uncovered.len(), 1);
    assert_eq!(uncovered[0].id, "12b1fb25cf755a9ce87c22a4");
}

#[test]
fn edit_distance_at_most_1_matches_single_edit() {
    assert!(edit_distance_at_most_1("abc", "abc"));
    assert!(edit_distance_at_most_1("abc", "ab"));
    assert!(edit_distance_at_most_1("abc", "abcd"));
    assert!(edit_distance_at_most_1("abc", "abd"));
    assert!(edit_distance_at_most_1("abc", "xbc"));
    assert!(!edit_distance_at_most_1("abc", "def"));
    assert!(!edit_distance_at_most_1("abc", "a"));
    assert!(!edit_distance_at_most_1("abc", "abde"));
    assert!(edit_distance_at_most_1("abc", "axc"));
}
