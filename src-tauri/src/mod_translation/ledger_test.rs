//! 任务状态中枢单元测试（经 ledger.rs 的 #[path] 子模块引入）

use super::*;

#[test]
fn upsert_dedupes_by_source_with_stable_id() {
    let mut graph = WorkGraph::new("task-1".to_string());
    let id_a = graph.upsert(
        WorkKind::Language,
        "翻译".to_string(),
        "assets/demo/lang/en_us.json#key".to_string(),
        1.0,
    );
    let id_b = graph.upsert(
        WorkKind::Language,
        "翻译（更新）".to_string(),
        "assets/demo/lang/en_us.json#key".to_string(),
        2.0,
    );
    assert_eq!(id_a, id_b);
    assert_eq!(id_a.len(), 24);
    assert_eq!(graph.items.len(), 1);
    assert_eq!(graph.items[&id_a].weight, 2.0);
    assert_eq!(graph.items[&id_a].version, 1);
}

#[test]
fn reset_for_retry_clears_model_attempts_keeps_quality_audit() {
    let mut graph = WorkGraph::new("task-2".to_string());
    let id = graph.upsert(
        WorkKind::Language,
        "翻译".to_string(),
        "src".to_string(),
        1.0,
    );
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
fn progress_sums_verified_weight() {
    let mut graph = WorkGraph::new("task-3".to_string());
    let id_a = graph.upsert(
        WorkKind::Language,
        "a".to_string(),
        "src-a".to_string(),
        1.0,
    );
    let id_b = graph.upsert(
        WorkKind::VisibleText,
        "b".to_string(),
        "src-b".to_string(),
        3.0,
    );
    graph.reconcile(&id_a, true, "ok");
    graph.supersede(&id_b, "merged");
    let (verified, total) = graph.progress();
    assert_eq!(verified, 1.0);
    assert_eq!(total, 4.0);
}

#[test]
fn supersede_is_irreversible() {
    let mut graph = WorkGraph::new("task-4".to_string());
    let id = graph.upsert(WorkKind::Language, "a".to_string(), "src".to_string(), 1.0);
    graph.supersede(&id, "replaced");
    graph.reconcile(&id, false, "retry");
    assert_eq!(graph.items[&id].status, WorkStatus::Superseded);
}

#[test]
fn task_memory_update_is_bounded() {
    let mut memory = TaskMemory::default();
    let glossary = (0..600)
        .map(|i| serde_json::json!({"source": format!("s{i}"), "translation": format!("t{i}")}))
        .collect::<Vec<_>>();
    let decisions = (0..300)
        .map(|i| serde_json::json!({"i": i}))
        .collect::<Vec<_>>();
    memory.update(serde_json::json!({
        "recommendedName": "x".repeat(100),
        "summary": "y".repeat(5000),
        "glossary": glossary,
        "decisions": decisions,
        "uncertainties": (0..200).map(|i| format!("u{i}")).collect::<Vec<_>>(),
        "discoveredTargets": (0..600).map(|i| format!("t{i}")).collect::<Vec<_>>(),
    }));
    assert_eq!(
        memory.recommended_name.as_ref().unwrap().chars().count(),
        80
    );
    assert_eq!(memory.summary.chars().count(), 4000);
    assert_eq!(memory.glossary.len(), 500);
    assert_eq!(memory.decisions.len(), 200);
    assert_eq!(memory.uncertainties.len(), 100);
    assert_eq!(memory.discovered_targets.len(), 500);
}
