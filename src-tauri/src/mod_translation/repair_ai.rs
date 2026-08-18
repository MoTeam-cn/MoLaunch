//! 质量回修：AI 修复方案请求与响应校验

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ai_core::{self, PromptKind};
use crate::mod_translation::prompt;
use crate::mod_translation::repair::{RepairAction, RepairIssue};
use crate::mod_translation::types::{has_chinese, ProgressFn, RetryInfo};

const MAX_ACTIONS_ATTEMPTS: u32 = 2;

pub(super) async fn request_actions(
    issues: &[RepairIssue],
    config: &ai_core::AiConfig,
    model: &str,
    on_progress: &ProgressFn,
    base_progress: f64,
    cancel: &AtomicBool,
) -> Result<Vec<RepairAction>, String> {
    let base = build_repair_prompt(issues);
    let mut last_error = None;
    for attempt in 0..MAX_ACTIONS_ATTEMPTS {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        if attempt > 0 {
            on_progress(
                base_progress,
                &format!("质量复验第 {}/{} 次重试", attempt + 1, MAX_ACTIONS_ATTEMPTS),
                Some(RetryInfo {
                    attempt: attempt + 1,
                    total: MAX_ACTIONS_ATTEMPTS,
                }),
            );
        }
        let user_content = match &last_error {
            Some(error) => format!("{base}\n上次输出校验失败：{error}。请完整重发合法 JSON。"),
            None => base.clone(),
        };
        let content = match ai_core::chat_json(
            config,
            PromptKind::ModTranslation,
            user_content,
            Some(model),
        )
        .await
        {
            Ok(content) => content,
            Err(e) => {
                last_error = Some(format!("AI 修复方案调用失败: {e}"));
                continue;
            }
        };
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

pub(crate) fn parse_actions_response(content: &str) -> Result<Vec<RepairAction>, String> {
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

pub(crate) fn validate_response(
    issues: &[RepairIssue],
    actions: &[RepairAction],
) -> Result<(), String> {
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
