//! 质量回修单元测试（经 repair.rs 的 #[path] 子模块引入）

use super::*;
use crate::mod_translation::ledger::{WorkGraph, WorkKind, WorkStatus};
use crate::mod_translation::types::{LanguageKind, LanguageSource};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static TEMP_SEQ: AtomicUsize = AtomicUsize::new(0);

fn temp_dir() -> PathBuf {
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "mo_launch_repair_test_{}_{seq}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets/demo/lang")).unwrap();
    dir
}

fn write_json(dir: &Path, rel: &str, map: &BTreeMap<String, String>) {
    let mut obj = serde_json::Map::new();
    for (k, v) in map {
        obj.insert(k.clone(), serde_json::Value::String(v.clone()));
    }
    std::fs::write(dir.join(rel), serde_json::Value::Object(obj).to_string()).unwrap();
}

fn json_source() -> LanguageSource {
    LanguageSource {
        kind: LanguageKind::Json,
        namespace: "demo".to_string(),
        source_path: "assets/demo/lang/en_us.json".to_string(),
        target_path: "assets/demo/lang/zh_cn.json".to_string(),
        entries: BTreeMap::from([
            ("missing.key".to_string(), "Missing text".to_string()),
            ("broken.key".to_string(), "Spawn %d zombies".to_string()),
            ("fine.key".to_string(), "Hello".to_string()),
        ]),
        existing_target: BTreeMap::new(),
    }
}

fn issue(id: &str, source: &str) -> RepairIssue {
    RepairIssue {
        id: id.to_string(),
        kind: "language".to_string(),
        target_path: Some("assets/demo/lang/zh_cn.json".to_string()),
        key: Some(format!("key.{id}")),
        source: source.to_string(),
        current: None,
        messages: vec!["缺少中文译文".to_string()],
        actionable: true,
    }
}

#[test]
fn collect_issues_aggregates_missing_and_placeholder_errors() {
    let dir = temp_dir();
    let source = json_source();
    write_json(&dir, &source.source_path, &source.entries);
    write_json(
        &dir,
        &source.target_path,
        &BTreeMap::from([
            ("broken.key".to_string(), "生成僵尸".to_string()),
            ("fine.key".to_string(), "你好".to_string()),
        ]),
    );
    let graph = WorkGraph::new("t".to_string());
    let issues = collect_issues(&dir, &[&source], &graph);
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().all(|i| i.id.len() == 16 && i.actionable));
    assert!(issues
        .iter()
        .any(|i| i.key.as_deref() == Some("missing.key")));
    assert!(issues
        .iter()
        .any(|i| i.messages.iter().any(|m| m.contains("占位符"))));
    // Superseded 条目被跳过
    let mut graph = WorkGraph::new("t".to_string());
    let wid = graph.upsert(
        WorkKind::Language,
        "g".to_string(),
        "assets/demo/lang/zh_cn.json#missing.key".to_string(),
        1.0,
    );
    graph.supersede(&wid, "保留");
    let issues = collect_issues(&dir, &[&source], &graph);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].key.as_deref(), Some("broken.key"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn parse_actions_response_strips_fences_and_extracts_actions() {
    let raw = "```json\n{\"actions\":[{\"action\":\"translate\",\"issueId\":\"abc\",\"translation\":\"你好\"},{\"action\":\"keep-source\",\"issueId\":\"def\",\"reason\":\"品牌名\"}]}\n```";
    let actions = parse_actions_response(raw).unwrap();
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0].action, "translate");
    assert_eq!(actions[0].translation.as_deref(), Some("你好"));
    assert_eq!(actions[1].action, "keep-source");
    assert!(parse_actions_response("无 JSON").is_err());
}

#[test]
fn validate_response_checks_coverage_duplicates_and_content() {
    let issues = vec![issue("a", "Spawn %d zombies"), issue("b", "Hello")];
    let good = vec![
        RepairAction {
            action: "translate".to_string(),
            issue_id: "a".to_string(),
            translation: Some("生成 %d 只僵尸".to_string()),
            reason: None,
        },
        RepairAction {
            action: "keep-source".to_string(),
            issue_id: "b".to_string(),
            translation: None,
            reason: Some("品牌名".to_string()),
        },
    ];
    let (validated, dropped) = validate_response(&issues, &good);
    assert!(!dropped && validated.len() == 2);
    // 未全覆盖：宽容处理补 keep-source，不报错
    let (validated, dropped) = validate_response(&issues, &good[..1]);
    assert!(dropped && validated.len() == 2);
    assert!(validated.iter().any(|a| a.action == "keep-source"));
    // 重复：丢弃重复项
    let dup = vec![good[0].clone(), good[0].clone()];
    let (validated, dropped) = validate_response(&issues, &dup);
    assert!(dropped && validated.len() == 2);
    // 译文无中文：丢弃该 action
    let latin = vec![
        RepairAction {
            action: "translate".to_string(),
            issue_id: "a".to_string(),
            translation: Some("Spawn %d zombies".to_string()),
            reason: None,
        },
        good[1].clone(),
    ];
    let (validated, dropped) = validate_response(&issues, &latin);
    assert!(dropped && validated.len() == 2);
    assert!(validated.iter().any(|a| a.action == "keep-source"));
    // keep-source 缺理由：丢弃该 action
    let no_reason = vec![
        good[0].clone(),
        RepairAction {
            action: "keep-source".to_string(),
            issue_id: "b".to_string(),
            translation: None,
            reason: None,
        },
    ];
    let (validated, dropped) = validate_response(&issues, &no_reason);
    assert!(dropped && validated.len() == 2);
}

#[test]
fn validate_response_accepts_truncated_issue_id() {
    let issues = vec![issue("abcdef1234567890", "Spawn %d zombies")];
    let actions = vec![RepairAction {
        action: "translate".to_string(),
        issue_id: "abcdef123456".to_string(), // 模型截断的 id
        translation: Some("生成 %d 只僵尸".to_string()),
        reason: None,
    }];
    let (validated, dropped) = validate_response(&issues, &actions);
    assert!(!dropped && validated.len() == 1);
    // 前缀不唯一时丢弃该 action，未覆盖项保留原文
    let issues = vec![
        issue("abcdef1234567890", "Spawn %d zombies"),
        issue("abcdef1234567891", "Hello"),
    ];
    let actions = vec![RepairAction {
        action: "translate".to_string(),
        issue_id: "abcdef123456".to_string(),
        translation: Some("生成 %d 只僵尸".to_string()),
        reason: None,
    }];
    let (validated, dropped) = validate_response(&issues, &actions);
    assert!(dropped && validated.len() == 2);
    assert!(validated.iter().all(|a| a.action == "keep-source"));
}

#[test]
fn apply_actions_writes_back_and_supersedes() {
    let dir = temp_dir();
    let source = LanguageSource {
        kind: LanguageKind::Json,
        namespace: "demo".to_string(),
        source_path: "assets/demo/lang/en_us.json".to_string(),
        target_path: "assets/demo/lang/zh_cn.json".to_string(),
        entries: BTreeMap::from([
            ("a.key".to_string(), "Spawn %d zombies".to_string()),
            ("b.key".to_string(), "Hello".to_string()),
        ]),
        existing_target: BTreeMap::new(),
    };
    write_json(&dir, &source.source_path, &source.entries);
    let mut graph = WorkGraph::new("t".to_string());
    let actions = vec![
        RepairAction {
            action: "translate".to_string(),
            issue_id: issue_id("language", &format!("{}#a.key", source.target_path)),
            translation: Some("生成 %d 只僵尸".to_string()),
            reason: None,
        },
        RepairAction {
            action: "keep-source".to_string(),
            issue_id: issue_id("language", &format!("{}#b.key", source.target_path)),
            translation: None,
            reason: Some("品牌名".to_string()),
        },
    ];
    apply_actions(&dir, &source, &actions, &mut graph).unwrap();
    let content = std::fs::read_to_string(dir.join(&source.target_path)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(value["a.key"], "生成 %d 只僵尸");
    assert_eq!(value["b.key"], "Hello");
    let item_b = graph
        .items
        .values()
        .find(|i| i.source == "assets/demo/lang/zh_cn.json#b.key")
        .unwrap();
    assert_eq!(item_b.status, WorkStatus::Superseded);
    let item_a = graph
        .items
        .values()
        .find(|i| i.source == "assets/demo/lang/zh_cn.json#a.key")
        .unwrap();
    assert!(item_a.attempts.iter().any(|t| t.action == "repair"));
    let _ = std::fs::remove_dir_all(&dir);
}
