//! class 翻译路由测试（经 translate_class.rs 的 #[path] 子模块引入）

use super::*;
use crate::mod_translation::ledger::ClassDecisionLedger;
use crate::mod_translation::types::{ClassCandidate, JarInspection, Loader, Quote};

fn candidate(id: &str, path: &str, text: &str) -> ClassCandidate {
    ClassCandidate {
        id: id.to_string(),
        path: path.to_string(),
        paths: vec![path.to_string()],
        occurrences: 1,
        text: text.to_string(),
    }
}

fn inspection_with(candidates: Vec<ClassCandidate>) -> JarInspection {
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

#[test]
fn test_deterministic_exclusions_skip_model_for_structural_text() {
    let inspection = inspection_with(vec![
        candidate("c1", "a/b/SomeClass.class", "com.example.SomeClass"),
        candidate(
            "c2",
            "xaero/map/gui/GuiMap.class",
            "Failed to load your map. Retry?",
        ),
    ]);
    let mut ledger = ClassDecisionLedger::default();

    resolve_deterministic_exclusions(&inspection, &mut ledger);

    assert_eq!(
        ledger
            .decision("c1")
            .map(|decision| decision.verdict.as_str()),
        Some("exclude")
    );
    assert_eq!(
        ledger
            .decision("c1")
            .and_then(|decision| decision.note.as_deref()),
        Some("java_class_name")
    );
    // 界面宿主路径保留，交给 AI 判定
    assert!(ledger.decision("c2").is_none());
    assert_eq!(ledger.pending(&inspection.class_candidates).len(), 1);
}

#[test]
fn test_parse_decisions_rejects_missing_array_and_non_json() {
    let candidates = [candidate("c1", "demo/A.class", "Iron Ingot")];
    let refs: Vec<&ClassCandidate> = candidates.iter().collect();
    assert!(parse_and_validate_decisions(r#"{"foo":1}"#, &refs).is_err());
    assert!(parse_and_validate_decisions("not json", &refs).is_err());
}

#[test]
fn test_parse_decisions_drops_invalid_and_reports_uncovered() {
    let candidates = [
        candidate("c1", "demo/A.class", "Iron Ingot"),
        candidate("c2", "demo/B.class", "Hello"),
    ];
    let refs: Vec<&ClassCandidate> = candidates.iter().collect();

    // 译文不含中文 → 丢弃，两条都未覆盖
    let bad = r#"{"decisions":[{"id":"c1","action":"translate","translation":"Iron Ingot","reason":"visible"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(bad, &refs).expect("parse");
    assert!(decisions.is_empty());
    assert_eq!(uncovered.len(), 2);

    // 只覆盖 c1
    let partial = r#"{"decisions":[{"id":"c1","action":"exclude","reason":"internal"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(partial, &refs).expect("parse");
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].id, "c1");
    assert_eq!(uncovered.len(), 1);
    assert_eq!(uncovered[0].id, "c2");

    // 未知 id
    let unknown = r#"{"decisions":[{"id":"nope","action":"exclude","reason":"internal"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(unknown, &refs).expect("parse");
    assert!(decisions.is_empty());
    assert_eq!(uncovered.len(), 2);

    // 重复 id → 只保留第一个
    let duplicated = r#"{"decisions":[{"id":"c1","action":"exclude","reason":"a"},{"id":"c1","action":"exclude","reason":"b"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(duplicated, &refs).expect("parse");
    assert_eq!(decisions.len(), 1);
    assert_eq!(uncovered.len(), 1);
    assert_eq!(uncovered[0].id, "c2");
}

#[test]
fn test_parse_decisions_accepts_valid_translate_and_rejects_placeholder_break() {
    let candidates = [candidate("c1", "demo/A.class", "Spawn %d zombies")];
    let refs: Vec<&ClassCandidate> = candidates.iter().collect();

    let ok = r#"{"decisions":[{"id":"c1","action":"translate","translation":"生成 %d 只僵尸","reason":"visible"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(ok, &refs).expect("parse");
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].translation.as_deref(), Some("生成 %d 只僵尸"));
    assert!(uncovered.is_empty());

    let mismatch = r#"{"decisions":[{"id":"c1","action":"translate","translation":"生成 %s 只僵尸","reason":"visible"}]}"#;
    let (decisions, uncovered) = parse_and_validate_decisions(mismatch, &refs).expect("parse");
    assert!(decisions.is_empty());
    assert_eq!(uncovered.len(), 1);
}

#[test]
fn test_parse_decisions_tolerates_single_edit_id() {
    let candidates = [
        candidate("037980176fd0aea1aebd7ab2", "demo/A.class", "Iron Ingot"),
        candidate("12b1fb25cf755a9ce87c22a4", "demo/B.class", "Hello"),
    ];
    let refs: Vec<&ClassCandidate> = candidates.iter().collect();
    let edited = r#"{"decisions":[{"id":"037980176fd0aea1ebd7ab2","action":"exclude","reason":"internal"}]}"#;

    let (decisions, uncovered) = parse_and_validate_decisions(edited, &refs).expect("parse");
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].id, "037980176fd0aea1aebd7ab2");
    assert_eq!(uncovered.len(), 1);
    assert_eq!(uncovered[0].id, "12b1fb25cf755a9ce87c22a4");
}

#[test]
fn test_edit_distance_matches_single_edit_only() {
    for (left, right) in [
        ("abc", "abc"),
        ("abc", "ab"),
        ("abc", "abcd"),
        ("abc", "abd"),
        ("abc", "xbc"),
        ("abc", "axc"),
    ] {
        assert!(edit_distance_at_most_1(left, right), "{left} vs {right}");
    }
    for (left, right) in [("abc", "def"), ("abc", "a"), ("abc", "abde")] {
        assert!(!edit_distance_at_most_1(left, right), "{left} vs {right}");
    }
}
