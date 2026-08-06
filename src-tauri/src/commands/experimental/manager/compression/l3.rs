//! L3 AI 语义摘要：调用轻量模型生成对话摘要
//!
//! 复用 `PromptKind::Summarize` 模板（summarize.md）+ `ai_core::chat`（自带超时），
//! 失败时降级返回 None（上层退化为 L1 截断 + 丢最旧）。

use crate::ai_core;
use crate::ai_core::prompt::PromptKind;
use crate::commands::experimental::types::{MessageItem, ToolCallRecord};
use crate::log_warn;

/// 拼接用于摘要的历史文本（含工具结果，保证摘要保留工具调用关键结论）
pub(super) fn history_for_summary(
    history: &[MessageItem],
    tool_records: &[ToolCallRecord],
) -> Vec<String> {
    let mut lines = Vec::new();
    for m in history {
        let content = m.content.as_str();
        lines.push(format!(
            "{}: {}",
            if m.role == "user" {
                "用户"
            } else if m.role == "assistant" {
                "助手"
            } else {
                m.role.as_str()
            },
            content
        ));
        if m.role == "assistant" {
            for r in tool_records.iter().filter(|r| r.message_id == m.id) {
                let out = r.output.as_deref().unwrap_or("（无输出）");
                lines.push(format!(
                    "→ 该回复前调用了工具 {}，结果：{}",
                    r.name,
                    crate::utils::format::truncate_chars(out, 600)
                ));
            }
        }
    }
    lines
}

/// 生成对话摘要（失败返回 None，不阻断主流程）
pub async fn summarize(
    config: &ai_core::AiConfig,
    model: &str,
    history: &[String],
) -> Option<String> {
    if history.is_empty() {
        return None;
    }
    // 拼接历史（每轮一行 role: content），截断保护避免超长
    let mut text = String::new();
    for line in history {
        let line = crate::utils::format::truncate_chars(line, 2000);
        text.push_str(&line);
        text.push('\n');
    }
    let text = crate::utils::format::truncate_chars(&text, 24_000);
    let user_content = format!("对话历史：\n{}", text);

    match ai_core::chat(config, PromptKind::Summarize, user_content, Some(model)).await {
        Ok(s) => {
            let s = s.trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
        Err(e) => {
            log_warn!("[Experimental] 压缩摘要生成失败（降级为 L1+丢最旧）: {}", e);
            None
        }
    }
}
