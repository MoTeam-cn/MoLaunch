//! 任务账本测试（经 ledger.rs 的 #[path] 子模块引入）

use super::*;

fn graph_with(kind: WorkKind, source: &str, weight: f64) -> (WorkGraph, String) {
    let mut graph = WorkGraph::new("task".to_string());
    let id = graph.upsert(kind, "翻译".to_string(), source.to_string(), weight);
    (graph, id)
}

#[test]
fn test_upsert_same_source_keeps_id_and_updates_weight() {
    let mut graph = WorkGraph::new("task-1".to_string());
    let first = graph.upsert(
        WorkKind::Language,
        "翻译".to_string(),
        "assets/demo/lang/en_us.json#key".to_string(),
        1.0,
    );
    let second = graph.upsert(
        WorkKind::Language,
        "翻译（更新）".to_string(),
        "assets/demo/lang/en_us.json#key".to_string(),
        2.0,
    );
    assert_eq!(first, second);
    assert_eq!(first.len(), 24);
    assert_eq!(graph.items.len(), 1);
    assert_eq!(graph.items[&first].weight, 2.0);
    assert_eq!(graph.items[&first].version, 1);
}

#[test]
fn test_reset_for_retry_clears_model_attempts_keeps_audit() {
    let (mut graph, id) = graph_with(WorkKind::Language, "src", 1.0);
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
fn test_progress_counts_only_verified_weight() {
    let mut graph = WorkGraph::new("task-3".to_string());
    let language = graph.upsert(
        WorkKind::Language,
        "a".to_string(),
        "src-a".to_string(),
        1.0,
    );
    let visible = graph.upsert(
        WorkKind::VisibleText,
        "b".to_string(),
        "src-b".to_string(),
        3.0,
    );
    graph.reconcile(&language, true, "ok");
    graph.supersede(&visible, "merged");

    assert_eq!(graph.progress(), (1.0, 4.0));
}

#[test]
fn test_supersede_survives_failed_reconcile() {
    let (mut graph, id) = graph_with(WorkKind::Language, "src", 1.0);
    graph.supersede(&id, "replaced");
    graph.reconcile(&id, false, "retry");
    assert_eq!(graph.items[&id].status, WorkStatus::Superseded);
}

#[test]
fn test_snapshot_round_trips() {
    let (graph, id) = graph_with(WorkKind::Language, "src", 2.0);
    let restored = WorkGraph::from_snapshot(graph.snapshot());
    assert_eq!(restored.task_id, "task");
    assert_eq!(restored.items[&id].weight, 2.0);
    assert_eq!(restored.revision, graph.revision);
}

#[test]
fn test_task_memory_update_is_bounded() {
    let mut memory = TaskMemory::default();
    let glossary: Vec<serde_json::Value> = (0..600)
        .map(|index| serde_json::json!({"source": format!("s{index}"), "translation": format!("t{index}")}))
        .collect();
    let decisions: Vec<serde_json::Value> = (0..300)
        .map(|index| serde_json::json!({"i": index}))
        .collect();
    memory.update(serde_json::json!({
        "recommendedName": "x".repeat(100),
        "summary": "y".repeat(5000),
        "glossary": glossary,
        "decisions": decisions,
        "uncertainties": (0..200).map(|i| format!("u{i}")).collect::<Vec<_>>(),
        "discoveredTargets": (0..600).map(|i| format!("t{i}")).collect::<Vec<_>>(),
    }));

    assert_eq!(
        memory
            .recommended_name
            .as_deref()
            .map(|name| name.chars().count()),
        Some(80)
    );
    assert_eq!(memory.summary.chars().count(), 4000);
    assert_eq!(memory.glossary.len(), 500);
    assert_eq!(memory.decisions.len(), 200);
    assert_eq!(memory.uncertainties.len(), 100);
    assert_eq!(memory.discovered_targets.len(), 500);
}

#[test]
fn test_class_decision_ledger_tracks_unresolved_and_exclusions() {
    use crate::mod_translation::types::ClassCandidate;

    let candidates = vec![
        ClassCandidate {
            id: "a".to_string(),
            path: "x/A.class".to_string(),
            paths: vec!["x/A.class".to_string()],
            occurrences: 1,
            text: "Hello".to_string(),
        },
        ClassCandidate {
            id: "b".to_string(),
            path: "x/B.class".to_string(),
            paths: vec!["x/B.class".to_string()],
            occurrences: 1,
            text: "World".to_string(),
        },
    ];
    let mut ledger = ClassDecisionLedger::default();
    ledger.record("a", VERDICT_EXCLUDE, None, Some("internal"));

    assert_eq!(ledger.pending(&candidates).len(), 1);
    assert_eq!(ledger.excluded_ids(), vec!["a".to_string()]);
    assert_eq!(
        ledger
            .decision("a")
            .map(|decision| decision.verdict.as_str()),
        Some(VERDICT_EXCLUDE)
    );
}
