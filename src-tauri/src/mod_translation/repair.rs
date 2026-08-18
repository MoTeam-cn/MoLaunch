//! 模组翻译：质量回修兜底（复验 → AI 修复方案 → 原子写回）

use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::atomic::Ordering;

use crate::ai_core::{self, PromptKind};
use crate::log_warn;
use sha2::{Digest, Sha256};

use super::lang;
use super::ledger::{WorkGraph, WorkKind, WorkStatus};
use super::prompt;
use super::quality::{audit_invariants, audit_semantic, AuditSeverity};
use super::types::{has_chinese, LanguageKind, LanguageSource};

const MAX_REPAIR_PASSES: usize = 4;
const MAX_REPAIR_BATCH: usize = 24;

#[derive(Debug, Clone)]
pub struct RepairIssue {
    pub id: String,
    pub kind: String,
    pub target_path: Option<String>,
    pub key: Option<String>,
    pub source: String,
    pub current: Option<String>,
    pub messages: Vec<String>,
    pub actionable: bool,
}

#[derive(Debug, Clone)]
pub struct RepairAction {
    pub action: String,
    pub issue_id: String,
    pub translation: Option<String>,
    pub reason: Option<String>,
}

pub fn collect_issues(
    workspace: &Path,
    sources: &[&LanguageSource],
    work_graph: &WorkGraph,
) -> Vec<RepairIssue> {
    let mut issues = Vec::new();
    for source in sources {
        let target = read_target_map(workspace, source);
        let mut by_key: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for audit in audit_invariants(&source.entries, &target)
            .into_iter()
            .chain(audit_semantic(&source.entries, &target))
        {
            if audit.severity == AuditSeverity::Error
                && !is_superseded(work_graph, &source.target_path, &audit.key)
            {
                by_key.entry(audit.key).or_default().push(audit.message);
            }
        }
        for (key, messages) in by_key {
            issues.push(RepairIssue {
                id: issue_id("language", &format!("{}#{}", source.target_path, key)),
                kind: "language".to_string(),
                target_path: Some(source.target_path.clone()),
                key: Some(key.clone()),
                source: source.entries.get(&key).cloned().unwrap_or_default(),
                current: target.get(&key).cloned(),
                messages,
                actionable: true,
            });
        }
    }
    issues.sort_by(|a, b| a.id.cmp(&b.id));
    issues
}

fn read_target_map(workspace: &Path, source: &LanguageSource) -> BTreeMap<String, String> {
    let Ok(content) = std::fs::read_to_string(workspace.join(&source.target_path)) else {
        return BTreeMap::new();
    };
    match source.kind {
        LanguageKind::Json => lang::read_json_lang(&content).unwrap_or_default(),
        LanguageKind::KeyValue => lang::parse_keyvalue(&content).0.into_iter().collect(),
        LanguageKind::StructuredJson => {
            lang::collect_structured_strings(&content).unwrap_or_default()
        }
        LanguageKind::FreeText => {
            lang::read_localized_target(workspace, Path::new(&source.target_path))
                .into_iter()
                .filter_map(|(p, t)| {
                    Some((
                        p.strip_prefix("/lines/")?
                            .parse::<usize>()
                            .ok()?
                            .to_string(),
                        t,
                    ))
                })
                .collect()
        }
    }
}

fn issue_id(kind: &str, source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

fn is_superseded(work_graph: &WorkGraph, target_path: &str, key: &str) -> bool {
    let source = format!("{target_path}#{key}");
    work_graph.items.values().any(|item| {
        item.kind == WorkKind::Language
            && item.source == source
            && item.status == WorkStatus::Superseded
    })
}

pub async fn request_actions(
    issues: &[RepairIssue],
    config: &ai_core::AiConfig,
    model: &str,
) -> Result<Vec<RepairAction>, String> {
    let base = build_repair_prompt(issues);
    let mut last_error = None;
    for _ in 0..2 {
        let user_content = match &last_error {
            Some(error) => format!("{base}\n上次输出校验失败：{error}。请完整重发合法 JSON。"),
            None => base.clone(),
        };
        let content = ai_core::chat_json(
            config,
            PromptKind::ModTranslation,
            user_content,
            Some(model),
        )
        .await
        .map_err(|e| format!("AI 修复方案调用失败: {e}"))?;
        match parse_actions_response(&content)
            .and_then(|actions| validate_response(issues, &actions).map(|_| actions))
        {
            Ok(actions) => return Ok(actions),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| "模型没有返回修复操作".to_string()))
}

fn build_repair_prompt(issues: &[RepairIssue]) -> String {
    let list: Vec<serde_json::Value> = issues
        .iter()
        .map(|issue| {
            serde_json::json!({"id": issue.id, "kind": issue.kind, "source": issue.source, "current": issue.current, "messages": issue.messages})
        })
        .collect();
    serde_json::json!({"task": "修复以下语言条目的质量问题，只输出 JSON 对象：{\"actions\":[{\"action\":\"translate\"|\"keep-source\",\"issueId\":\"...\",\"translation\":\"...\",\"reason\":\"...\"}]}", "rules": "translate 必须含简体中文并保留全部占位符；确实应保留原文时用 keep-source 并给出 reason。每个 issue 恰好一个 action。issueId 必须原样复制 issues 中的 id，不得修改、截断或自行生成。", "issues": list})
        .to_string()
}

fn str_field(item: &serde_json::Value, name: &str) -> Option<String> {
    item.get(name)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn parse_actions_response(content: &str) -> Result<Vec<RepairAction>, String> {
    let stripped = prompt::strip_json_fences(content);
    let start = stripped.find('{').ok_or("AI 响应中未找到 JSON 对象")?;
    let end = stripped.rfind('}').ok_or("AI 响应中未找到 JSON 对象")?;
    let value: serde_json::Value = serde_json::from_str(&stripped[start..=end])
        .map_err(|e| format!("解析修复 JSON 失败: {e}"))?;
    let actions = value
        .get("actions")
        .and_then(serde_json::Value::as_array)
        .ok_or("AI 响应缺少 actions 数组")?;
    let mut result = Vec::new();
    for item in actions {
        let action = str_field(item, "action").unwrap_or_default();
        let issue_id = str_field(item, "issueId").unwrap_or_default();
        if action.is_empty() || issue_id.is_empty() {
            return Err("action 缺少 action/issueId 字段".to_string());
        }
        result.push(RepairAction {
            action,
            issue_id,
            translation: str_field(item, "translation"),
            reason: str_field(item, "reason"),
        });
    }
    Ok(result)
}

/// 模型可能截断/改写 issueId：先精确匹配，再按前缀唯一匹配兜底
fn resolve_issue_id<'a>(expected: &'a HashSet<&str>, raw: &str) -> Option<&'a str> {
    if expected.contains(raw) {
        return expected.iter().copied().find(|id| *id == raw);
    }
    let mut matches = expected.iter().filter(|id| id.starts_with(raw));
    let first = matches.next()?;
    if matches.next().is_some() {
        return None; // 前缀不唯一，无法确定
    }
    Some(*first)
}

fn validate_response(issues: &[RepairIssue], actions: &[RepairAction]) -> Result<(), String> {
    let expected: HashSet<&str> = issues.iter().map(|issue| issue.id.as_str()).collect();
    let mut seen = HashSet::new();
    for action in actions {
        let Some(issue_id) = resolve_issue_id(&expected, &action.issue_id) else {
            return Err(format!("模型返回未知 issue：{}", action.issue_id));
        };
        if !seen.insert(issue_id) {
            return Err(format!("模型重复返回 issue：{issue_id}"));
        }
        let issue = issues.iter().find(|i| i.id == issue_id).unwrap();
        let label = issue.key.as_deref().unwrap_or(action.issue_id.as_str());
        match action.action.as_str() {
            "translate" => {
                let t = action.translation.as_deref().unwrap_or_default();
                if !has_chinese(t) || !prompt::validate_translation(&issue.source, t) {
                    return Err(format!("条目 {label} 的译文不含中文或占位符不一致"));
                }
            }
            "keep-source" => {
                let reason = action.reason.as_deref().unwrap_or_default();
                if reason.trim().is_empty() {
                    return Err(format!("条目 {label} 的 keep-source 缺少理由"));
                }
            }
            other => return Err(format!("未知 action 类型：{other}")),
        }
    }
    if seen.len() != expected.len() {
        return Err(format!(
            "模型只处理了 {}/{} 个 issue",
            seen.len(),
            expected.len()
        ));
    }
    Ok(())
}

pub fn apply_actions(
    workspace: &Path,
    source: &LanguageSource,
    actions: &[RepairAction],
    work_graph: &mut WorkGraph,
) -> Result<(), String> {
    let mut translations = BTreeMap::new();
    for action in actions {
        let key = action_key(source, &action.issue_id)?;
        match action.action.as_str() {
            "translate" => {
                let translation = action.translation.as_deref().unwrap_or_default();
                translations.insert(key, translation.to_string());
            }
            "keep-source" => {
                let text = source.entries.get(&key).cloned().unwrap_or_default();
                translations.insert(key, text);
            }
            other => return Err(format!("未知 action 类型：{other}")),
        }
    }
    if !translations.is_empty() {
        write_target(workspace, source, &translations)?;
    }
    for action in actions {
        let key = action_key(source, &action.issue_id)?;
        let item_id = work_graph.upsert(
            WorkKind::Language,
            "翻译".to_string(),
            format!("{}#{}", source.target_path, key),
            1.0,
        );
        if action.action == "keep-source" {
            work_graph.supersede(&item_id, "显式保留原文");
        }
        work_graph.record_attempt(&item_id, "repair".to_string(), "ok".to_string(), None);
    }
    Ok(())
}

fn action_key(source: &LanguageSource, want_id: &str) -> Result<String, String> {
    for key in source.entries.keys() {
        if issue_id("language", &format!("{}#{}", source.target_path, key)) == want_id {
            return Ok(key.clone());
        }
    }
    Err(format!("未知 issue：{want_id}"))
}

fn write_target(
    workspace: &Path,
    source: &LanguageSource,
    translations: &BTreeMap<String, String>,
) -> Result<(), String> {
    let source_path = workspace.join(&source.source_path);
    let target_path = workspace.join(&source.target_path);
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("无法创建目标目录: {e}"))?;
    }
    let original = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("无法读取 {}: {e}", source_path.display()))?;
    let output = match source.kind {
        LanguageKind::Json => lang::write_json_lang(&original, translations)?,
        LanguageKind::KeyValue => {
            lang::write_keyvalue(&lang::parse_keyvalue(&original).1, translations)
        }
        LanguageKind::StructuredJson => lang::apply_structured_strings(&original, translations)?,
        LanguageKind::FreeText => {
            let mut snap = lang::snapshot_free_text(&original);
            snap.target_lines = lang::align_free_text(&snap.source_lines, translations);
            lang::render_localized_text(&snap)
        }
    };
    let tmp = target_path.with_extension("tmp");
    std::fs::write(&tmp, output).map_err(|e| format!("无法写入 {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &target_path)
        .map_err(|e| format!("无法替换 {}: {e}", target_path.display()))
}

pub async fn run_repair_passes(
    workspace: &Path,
    sources: &[&LanguageSource],
    work_graph: &mut WorkGraph,
    config: &ai_core::AiConfig,
    model: &str,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<bool, String> {
    for _ in 0..MAX_REPAIR_PASSES {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        let issues = collect_issues(workspace, sources, work_graph);
        if issues.is_empty() {
            return Ok(true);
        }
        let mut groups: BTreeMap<String, Vec<RepairIssue>> = BTreeMap::new();
        for issue in issues {
            let path = issue.target_path.clone().unwrap_or_default();
            groups.entry(path).or_default().push(issue);
        }
        for (target_path, group) in groups {
            if cancel.load(Ordering::Relaxed) {
                return Err("任务已取消".to_string());
            }
            let source = sources
                .iter()
                .find(|s| s.target_path == target_path)
                .ok_or_else(|| format!("未知语言目标：{target_path}"))?;
            for batch in group.chunks(MAX_REPAIR_BATCH) {
                // 回修是兜底：批次失败仅跳过，不阻塞打包
                match request_actions(batch, config, model).await {
                    Ok(actions) => {
                        if let Err(e) = apply_actions(workspace, source, &actions, work_graph) {
                            log_warn!("[ModTranslation] 质量回修写回失败，跳过该批次: {e}");
                        }
                    }
                    Err(e) => log_warn!("[ModTranslation] 质量回修批次失败，跳过: {e}"),
                }
            }
        }
    }
    let remaining = collect_issues(workspace, sources, work_graph);
    Ok(remaining.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("mo_launch_repair_test_{}", std::process::id()));
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
        assert!(validate_response(&issues, &good).is_ok());
        assert!(validate_response(&issues, &good[..1]).is_err()); // 未全覆盖
        let dup = vec![good[0].clone(), good[0].clone()];
        assert!(validate_response(&issues, &dup).is_err()); // 重复
        let latin = vec![
            RepairAction {
                action: "translate".to_string(),
                issue_id: "a".to_string(),
                translation: Some("Spawn %d zombies".to_string()),
                reason: None,
            },
            good[1].clone(),
        ];
        assert!(validate_response(&issues, &latin).is_err()); // 译文无中文
        let no_reason = vec![
            good[0].clone(),
            RepairAction {
                action: "keep-source".to_string(),
                issue_id: "b".to_string(),
                translation: None,
                reason: None,
            },
        ];
        assert!(validate_response(&issues, &no_reason).is_err()); // keep-source 缺理由
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
        assert!(validate_response(&issues, &actions).is_ok());
        // 前缀不唯一时仍拒绝
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
        assert!(validate_response(&issues, &actions).is_err());
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
}
