//! 质量回修测试（经 repair.rs 的 #[path] 子模块引入）

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::*;
use crate::mod_translation::ledger::WorkStatus;

const SOURCE_PATH: &str = "assets/demo/lang/en_us.json";
const TARGET_PATH: &str = "assets/demo/lang/zh_cn.json";

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!("molaunch-repair-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_json(dir: &Path, relative: &str, entries: &BTreeMap<String, String>) {
    let path = dir.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(&path, serde_json::to_string(entries).expect("serialize")).expect("write json");
}

fn json_source() -> LanguageSource {
    LanguageSource {
        kind: LanguageKind::Json,
        namespace: "demo".to_string(),
        source_path: SOURCE_PATH.to_string(),
        target_path: TARGET_PATH.to_string(),
        entries: BTreeMap::from([
            ("missing.key".to_string(), "Hello".to_string()),
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
        target_path: Some(TARGET_PATH.to_string()),
        key: Some(format!("key.{id}")),
        source: source.to_string(),
        current: None,
        messages: vec!["缺少中文译文".to_string()],
        actionable: true,
    }
}

fn action(
    action: &str,
    issue_id: &str,
    translation: Option<&str>,
    reason: Option<&str>,
) -> RepairAction {
    RepairAction {
        action: action.to_string(),
        issue_id: issue_id.to_string(),
        translation: translation.map(str::to_string),
        reason: reason.map(str::to_string),
    }
}

#[test]
fn test_collect_issues_reports_missing_and_placeholder_errors() {
    let dir = temp_dir("collect");
    let source = json_source();
    write_json(&dir, SOURCE_PATH, &source.entries);
    write_json(
        &dir,
        TARGET_PATH,
        &BTreeMap::from([
            ("broken.key".to_string(), "生成僵尸".to_string()),
            ("fine.key".to_string(), "你好".to_string()),
        ]),
    );

    let graph = WorkGraph::new("t".to_string());
    let issues = collect_issues(&dir, &[&source], &graph);
    assert_eq!(issues.len(), 2);
    assert!(issues
        .iter()
        .all(|issue| issue.id.len() == 16 && issue.actionable));
    assert!(issues
        .iter()
        .any(|issue| issue.key.as_deref() == Some("missing.key")));
    assert!(issues.iter().any(|issue| issue
        .messages
        .iter()
        .any(|message| message.contains("占位符"))));

    // Superseded 条目不再回修
    let mut graph = WorkGraph::new("t".to_string());
    let work_id = graph.upsert(
        WorkKind::Language,
        "g".to_string(),
        format!("{TARGET_PATH}#missing.key"),
        1.0,
    );
    graph.supersede(&work_id, "保留");
    let issues = collect_issues(&dir, &[&source], &graph);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].key.as_deref(), Some("broken.key"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_parse_actions_response_strips_fences() {
    let raw = "```json\n{\"actions\":[{\"action\":\"translate\",\"issueId\":\"abc\",\"translation\":\"你好\"},{\"action\":\"keep-source\",\"issueId\":\"def\",\"reason\":\"品牌名\"}]}\n```";
    let actions = parse_actions_response(raw).expect("parse");
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0].action, "translate");
    assert_eq!(actions[0].translation.as_deref(), Some("你好"));
    assert_eq!(actions[1].action, "keep-source");
    assert!(parse_actions_response("无 JSON").is_err());
}

#[test]
fn test_validate_response_covers_partial_duplicate_and_invalid_cases() {
    let issues = vec![issue("a", "Spawn %d zombies"), issue("b", "Hello")];
    let good = vec![
        action("translate", "a", Some("生成 %d 只僵尸"), None),
        action("keep-source", "b", None, Some("品牌名")),
    ];

    let (validated, dropped) = validate_response(&issues, &good);
    assert!(!dropped);
    assert_eq!(validated.len(), 2);

    // 未全覆盖：补 keep-source
    let (validated, dropped) = validate_response(&issues, &good[..1]);
    assert!(dropped);
    assert_eq!(validated.len(), 2);
    assert!(validated.iter().any(|entry| entry.action == "keep-source"));

    // 重复：丢弃重复项
    let duplicated = vec![good[0].clone(), good[0].clone()];
    let (validated, dropped) = validate_response(&issues, &duplicated);
    assert!(dropped);
    assert_eq!(validated.len(), 2);

    // 译文无中文：丢弃该动作并保留原文
    let latin = vec![
        action("translate", "a", Some("Spawn %d zombies"), None),
        good[1].clone(),
    ];
    let (validated, dropped) = validate_response(&issues, &latin);
    assert!(dropped);
    assert_eq!(validated.len(), 2);
    assert!(validated.iter().any(|entry| entry.action == "keep-source"));

    // keep-source 缺理由：丢弃该动作
    let no_reason = vec![good[0].clone(), action("keep-source", "b", None, None)];
    let (validated, dropped) = validate_response(&issues, &no_reason);
    assert!(dropped);
    assert_eq!(validated.len(), 2);
}

#[test]
fn test_validate_response_accepts_truncated_id_when_unique() {
    let issues = vec![issue("abcdef1234567890", "Spawn %d zombies")];
    let actions = vec![action(
        "translate",
        "abcdef123456",
        Some("生成 %d 只僵尸"),
        None,
    )];
    let (validated, dropped) = validate_response(&issues, &actions);
    assert!(!dropped);
    assert_eq!(validated.len(), 1);
    assert_eq!(validated[0].issue_id, "abcdef1234567890");

    // 前缀不唯一时丢弃该动作，两条都保留原文
    let issues = vec![
        issue("abcdef1234567890", "Spawn %d zombies"),
        issue("abcdef1234567891", "Hello"),
    ];
    let (validated, dropped) = validate_response(&issues, &actions);
    assert!(dropped);
    assert_eq!(validated.len(), 2);
    assert!(validated.iter().all(|entry| entry.action == "keep-source"));
}

#[test]
fn test_apply_actions_writes_back_and_supersedes() {
    let dir = temp_dir("apply");
    let source = LanguageSource {
        kind: LanguageKind::Json,
        namespace: "demo".to_string(),
        source_path: SOURCE_PATH.to_string(),
        target_path: TARGET_PATH.to_string(),
        entries: BTreeMap::from([
            ("a.key".to_string(), "Spawn %d zombies".to_string()),
            ("b.key".to_string(), "Hello".to_string()),
        ]),
        existing_target: BTreeMap::new(),
    };
    write_json(&dir, SOURCE_PATH, &source.entries);

    let id_a = issue_id("language", &format!("{TARGET_PATH}#a.key"));
    let id_b = issue_id("language", &format!("{TARGET_PATH}#b.key"));
    let actions = vec![
        action("translate", &id_a, Some("生成 %d 只僵尸"), None),
        action("keep-source", &id_b, None, Some("品牌名")),
    ];

    let mut graph = WorkGraph::new("task".to_string());
    apply_actions(&dir, &source, &actions, &mut graph).expect("apply");

    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join(TARGET_PATH)).expect("read target"))
            .expect("parse target");
    assert_eq!(written["a.key"], "生成 %d 只僵尸");
    assert_eq!(written["b.key"], "Hello");

    let item_a = graph
        .items
        .values()
        .find(|item| item.source == format!("{TARGET_PATH}#a.key"))
        .expect("item a");
    assert!(item_a
        .attempts
        .iter()
        .any(|attempt| attempt.action == "repair"));
    let item_b = graph
        .items
        .values()
        .find(|item| item.source == format!("{TARGET_PATH}#b.key"))
        .expect("item b");
    assert_eq!(item_b.status, WorkStatus::Superseded);

    let _ = std::fs::remove_dir_all(&dir);
}
