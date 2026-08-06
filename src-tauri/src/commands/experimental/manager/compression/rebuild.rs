//! 上下文重塑器：组装压缩后的 turns（摘要 + 边界标记 + 最近 N 条 + 工具轮次）
//!
//! 工具轮次以文本块注入对应 assistant 消息后（模型可见工具与结果），
//! 规避 OpenAI 兼容服务对「assistant(tool_calls) 后必须紧跟 tool 消息」的严格配对要求。

use crate::ai_core::client::ChatTurn;
use crate::commands::experimental::types::{MessageItem, ToolCallRecord};

/// 边界后保留的原始消息条数（短期工作记忆）
pub(super) const RECENT_KEEP: usize = 15;
/// 边界标记文本（隔离压缩前内容）
pub(super) const BOUNDARY_MARKER: &str =
    "[COMPACT_BOUNDARY] --- 以下为压缩后的上下文，之前的详细对话已总结为上方摘要 ---";

/// 聊天全局系统提示词 turn（压缩/未压缩路径统一插入在最前）
pub(super) fn system_turn() -> ChatTurn {
    ChatTurn::plain("system", crate::ai_core::prompt::chat_system_prompt())
}

/// 注入边界标记（摘要与最近消息之间）
fn push_boundary(turns: &mut Vec<ChatTurn>) {
    turns.push(ChatTurn::plain("system", BOUNDARY_MARKER.to_string()));
}

/// 注入摘要（位于最前，紧随系统提示词）
fn push_summary(turns: &mut Vec<ChatTurn>, summary: &str) {
    turns.push(ChatTurn::plain(
        "system",
        format!("【压缩摘要】\n{}", summary),
    ));
}

/// 按消息分组工具调用记录（assistant 消息 → 其触发的工具轮次）
fn group_tools(records: &[ToolCallRecord]) -> std::collections::HashMap<i64, Vec<&ToolCallRecord>> {
    let mut map: std::collections::HashMap<i64, Vec<&ToolCallRecord>> =
        std::collections::HashMap::new();
    for r in records {
        map.entry(r.message_id).or_default().push(r);
    }
    map
}

/// 将工具调用记录以文本块注入对应 assistant 消息的 turns 中
///
/// `turns` 必须与 `history` 一一对应（由同一来源构造、无系统提示词偏移），
/// 供压缩（rebuild）与未压缩（pipeline 直构）两条路径复用。
pub(super) fn inject_tool_blocks(
    turns: &mut [ChatTurn],
    history: &[MessageItem],
    tool_records: &[ToolCallRecord],
) {
    let by_msg = group_tools(tool_records);
    for (idx, m) in history.iter().enumerate() {
        if m.role != "assistant" {
            continue;
        }
        let Some(records) = by_msg.get(&m.id) else {
            continue;
        };
        if records.is_empty() {
            continue;
        }
        let mut block = String::new();
        for r in records {
            block.push_str(&format!("\n\n【工具调用：{}】\n", r.name));
            if let Some(pc) = r.pre_content.as_deref().filter(|s| !s.trim().is_empty()) {
                block.push_str(&format!("（模型说明：{}）\n", pc));
            }
            block.push_str(r.output.as_deref().unwrap_or("（无输出）"));
        }
        if let Some(t) = turns.get_mut(idx) {
            if let Some(c) = t.content.as_mut() {
                c.push_str(&block);
            }
        }
    }
}

/// 构造压缩后 turns：
///
/// `[摘要 system] → [边界 system] → 最近 RECENT_KEEP 条原始消息（含工具轮次文本）`
///
/// 注意：不包含聊天系统提示词，由调用方（pipeline）在最前插入。
pub fn rebuild(
    history: &[MessageItem],
    tool_records: &[ToolCallRecord],
    summary: Option<&str>,
) -> Vec<ChatTurn> {
    let mut turns: Vec<ChatTurn> = Vec::new();
    if let Some(s) = summary {
        push_summary(&mut turns, s);
    }
    push_boundary(&mut turns);

    // 仅保留最近 RECENT_KEEP 条原始消息（短期工作记忆）
    let recent: &[MessageItem] = if history.len() > RECENT_KEEP {
        &history[history.len() - RECENT_KEEP..]
    } else {
        history
    };

    let mut base: Vec<ChatTurn> = recent
        .iter()
        .map(|m| ChatTurn::plain(m.role.clone(), m.content.clone()))
        .collect();
    inject_tool_blocks(&mut base, recent, tool_records);
    turns.extend(base);
    turns
}
