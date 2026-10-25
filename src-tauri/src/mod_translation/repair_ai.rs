//! 质量回修：AI 修复方案请求与响应校验
//!
//! 回修是兜底流程：模型输出不完整时一律「丢弃无效项 + 未覆盖项保留原文」，
//! 只把可用动作交给写回层，绝不让回修本身阻塞打包。

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ai_core::{self, PromptKind};
use crate::log_warn;
use crate::mod_translation::prompt;
use crate::mod_translation::repair::{RepairAction, RepairIssue};
use crate::mod_translation::types::{has_chinese, ProgressFn, RetryInfo};

const MAX_ACTIONS_ATTEMPTS: u32 = 2;
const TRANSLATE_ACTION: &str = "translate";
const KEEP_SOURCE_ACTION: &str = "keep-source";

pub(super) async fn request_actions(
    issues: &[RepairIssue],
    config: &ai_core::AiConfig,
    model: &str,
    on_progress: &ProgressFn,
    base_progress: f64,
    cap_progress: f64,
    cancel: &AtomicBool,
) -> Result<Vec<RepairAction>, String> {
    let prompt_base = build_repair_prompt(issues);
    let mut last_error: Option<String> = None;

    for attempt in 0..MAX_ACTIONS_ATTEMPTS {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        let retry = (attempt > 0).then(|| RetryInfo {
            attempt: attempt + 1,
            total: MAX_ACTIONS_ATTEMPTS,
        });
        if attempt > 0 {
            log_warn!(
                "[ModTranslation] 质量复验第 {}/{} 次重试",
                attempt + 1,
                MAX_ACTIONS_ATTEMPTS
            );
        }

        let user_content = match &last_error {
            Some(error) => {
                format!("{prompt_base}\n上次输出校验失败：{error}。请完整重发合法 JSON。")
            }
            None => prompt_base.clone(),
        };
        let label = if attempt > 0 {
            format!("质量复验第 {}/{} 次重试", attempt + 1, MAX_ACTIONS_ATTEMPTS)
        } else {
            "质量复验中".to_string()
        };

        let content = match tokio::select! {
            result = ai_core::chat_json(
                config,
                PromptKind::ModTranslation,
                user_content,
                Some(model),
                Some(crate::mod_translation::progress::AI_TIMEOUT_SECS),
            ) => result,
            _ = crate::mod_translation::progress::wait_cancel(cancel) => {
                return Err("任务已取消".to_string())
            }
            _ = crate::mod_translation::progress::smooth_progress(
                "repair",
                base_progress,
                cap_progress,
                cancel,
                on_progress,
                move |_p: f64| label,
                retry,
            ) => return Err("任务已取消".to_string()),
        } {
            Ok(content) => content,
            Err(error) => {
                let message = format!("AI 修复方案调用失败: {error}");
                log_warn!("[ModTranslation] {message}");
                let current = crate::mod_translation::progress::current_stage_progress("repair");
                on_progress(current.max(base_progress), &message, retry);
                last_error = Some(message);
                continue;
            }
        };

        match parse_actions_response(&content) {
            Ok(actions) => {
                let (validated, dropped) = validate_response(issues, &actions);
                if dropped {
                    log_warn!("[ModTranslation] 修复方案不完整，未覆盖/无效的 issue 保留原文");
                }
                return Ok(validated);
            }
            Err(error) => {
                let message = format!("修复方案校验失败: {error}");
                log_warn!("[ModTranslation] {message}");
                let current = crate::mod_translation::current_stage_progress("repair");
                on_progress(current.max(base_progress), &message, retry);
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "模型没有返回修复操作".to_string()))
}

fn build_repair_prompt(issues: &[RepairIssue]) -> String {
    let list: Vec<serde_json::Value> = issues
        .iter()
        .map(|issue| {
            serde_json::json!({
                "id": issue.id,
                "kind": issue.kind,
                "source": issue.source,
                "current": issue.current,
                "messages": issue.messages,
            })
        })
        .collect();
    let total = issues.len();
    serde_json::json!({
        "task": "修复以下语言条目的质量问题，只输出 JSON 对象：{\"actions\":[{\"action\":\"translate\"|\"keep-source\",\"issueId\":\"...\",\"translation\":\"...\",\"reason\":\"...\"}]}。issues 数组包含待修复条目，数量非零，必须逐一处理。",
        "rules": format!(
            "translate 必须含简体中文并保留全部占位符；确实应保留原文时用 keep-source 并给出 reason。必须为 issues 中每一个 id 输出恰好一个 action，覆盖全部 {total} 个 issue，不得遗漏任何一个。issueId 必须原样复制 issues 中的 id，不得修改、截断或自行生成。输出精简：keep-source 时省略 translation 字段，reason 一句话即可。\n\n【严重警告】issues 数组包含 {total} 个条目，非空且必须全部处理。禁止输出空 actions 数组，禁止声称 issues 为空、不存在或无法处理，禁止忽略任何条目。若输出空 actions 或遗漏条目，本次修复将被判定为失败并重试。"
        ),
        "issues": list
    })
    .to_string()
}

fn str_field(item: &serde_json::Value, name: &str) -> Option<String> {
    item.get(name)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

/// 解析模型响应里的 actions 数组
pub(crate) fn parse_actions_response(content: &str) -> Result<Vec<RepairAction>, String> {
    let stripped = prompt::strip_json_fences(content);
    let json_text = prompt::extract_json_object(stripped).ok_or("AI 响应中未找到 JSON 对象")?;
    let value: serde_json::Value =
        serde_json::from_str(json_text).map_err(|e| format!("解析修复 JSON 失败: {e}"))?;
    let actions = value
        .get("actions")
        .and_then(serde_json::Value::as_array)
        .ok_or("AI 响应缺少 actions 数组")?;

    let mut parsed = Vec::with_capacity(actions.len());
    for item in actions {
        let action = str_field(item, "action").unwrap_or_default();
        let issue_id = str_field(item, "issueId").unwrap_or_default();
        if action.is_empty() || issue_id.is_empty() {
            return Err("action 缺少 action/issueId 字段".to_string());
        }
        parsed.push(RepairAction {
            action,
            issue_id,
            translation: str_field(item, "translation"),
            reason: str_field(item, "reason"),
        });
    }
    Ok(parsed)
}

/// 模型可能截断 issueId：精确匹配优先，其次唯一前缀匹配
fn match_issue_id<'a>(known: &'a HashSet<&str>, raw: &str) -> Option<&'a str> {
    if let Some(exact) = known.iter().copied().find(|id| *id == raw) {
        return Some(exact);
    }
    let mut candidates = known.iter().copied().filter(|id| id.starts_with(raw));
    let first = candidates.next()?;
    candidates.next().is_none().then_some(first)
}

/// 单条动作是否可采纳
fn action_is_valid(action: &RepairAction, issue: &RepairIssue) -> bool {
    match action.action.as_str() {
        TRANSLATE_ACTION => {
            let translation = action.translation.as_deref().unwrap_or_default();
            has_chinese(translation) && prompt::validate_translation(&issue.source, translation)
        }
        KEEP_SOURCE_ACTION => !action
            .reason
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty(),
        _ => false,
    }
}

/// 宽容校验：丢弃未知 / 重复 / 无效动作，未覆盖的 issue 自动保留原文
pub(crate) fn validate_response(
    issues: &[RepairIssue],
    actions: &[RepairAction],
) -> (Vec<RepairAction>, bool) {
    let known: HashSet<&str> = issues.iter().map(|issue| issue.id.as_str()).collect();
    let by_id: HashMap<&str, &RepairIssue> = issues
        .iter()
        .map(|issue| (issue.id.as_str(), issue))
        .collect();

    let mut seen: HashSet<&str> = HashSet::new();
    let mut accepted: Vec<RepairAction> = Vec::new();
    let mut dropped = false;

    for action in actions {
        let Some(issue_id) = match_issue_id(&known, &action.issue_id) else {
            dropped = true;
            continue;
        };
        if !seen.insert(issue_id) {
            dropped = true;
            continue;
        }
        let Some(issue) = by_id.get(issue_id) else {
            dropped = true;
            continue;
        };
        if !action_is_valid(action, issue) {
            let label = issue.key.as_deref().unwrap_or(action.issue_id.as_str());
            log_warn!("[ModTranslation] 丢弃无效修复 action：{label}");
            dropped = true;
            continue;
        }
        let mut accepted_action = action.clone();
        accepted_action.issue_id = issue_id.to_string();
        accepted.push(accepted_action);
    }

    for issue in issues {
        if seen.contains(issue.id.as_str()) {
            continue;
        }
        dropped = true;
        accepted.push(RepairAction {
            action: KEEP_SOURCE_ACTION.to_string(),
            issue_id: issue.id.clone(),
            translation: None,
            reason: Some("模型未处理，保留原文".to_string()),
        });
    }

    (accepted, dropped)
}
