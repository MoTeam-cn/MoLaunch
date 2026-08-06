//! 压缩管线总控：触发判定 → L1 截断 →（可选）L3 摘要 → 重塑 → 持久化
//!
//! `compact_if_needed` 为唯一对外入口，聊天三入口（chat/regenerate/edit）统一调用。
//! 仅统计历史窗口内消息绑定的工具记录（窗口外旧记录不参与触发与注入）。

use crate::ai_core;
use crate::ai_core::client::ChatTurn;
use crate::commands::experimental::db;
use crate::commands::experimental::types::{MessageItem, ToolCallRecord};

use super::super::context;
use super::l1;
use super::l3;
use super::rebuild;
use super::trigger;

/// 压缩结果信息（供上层 emit 提示）
pub struct CompactInfo {
    /// 是否发生了压缩
    pub compacted: bool,
    /// 触发原因/提示文案（空表示未压缩）
    pub reason: String,
    /// 摘要是否由 L3 生成（false 表示退化为 L1+丢最旧）
    pub has_summary: bool,
}

/// 构造带压缩的上下文 turns（含系统提示词）
///
/// 触发判定通过时执行压缩管线；未触发时直接按全部历史构造 turns
/// （同时注入工具轮次文本，保证历史上下文含真实工具结果）。
pub async fn compact_if_needed(
    config: &ai_core::AiConfig,
    model: &str,
    conversation_id: i64,
    history: &[MessageItem],
    tool_records: &[ToolCallRecord],
) -> Result<(Vec<ChatTurn>, CompactInfo), String> {
    // 仅统计历史窗口内消息绑定的工具记录（触发判定与注入均基于有效窗口）
    let scope_ids: std::collections::HashSet<i64> = history.iter().map(|m| m.id).collect();
    let scope_records: Vec<ToolCallRecord> = tool_records
        .iter()
        .filter(|r| scope_ids.contains(&r.message_id))
        .cloned()
        .collect();

    let decision = trigger::evaluate(
        conversation_id,
        history,
        &scope_records,
        config.max_input_tokens,
    );

    if !decision.should_compact {
        // 未压缩：直构全部历史 + 注入工具轮次（turns 与 history 一一对应，再补系统提示词）
        let mut turns = context::build_turns(history, false);
        rebuild::inject_tool_blocks(&mut turns, history, &scope_records);
        turns.insert(0, rebuild::system_turn());
        return Ok((
            turns,
            CompactInfo {
                compacted: false,
                reason: String::new(),
                has_summary: false,
            },
        ));
    }

    trigger::record_compaction(conversation_id);

    // L1: 工具输出截断（避免巨型输出再次挤爆新上下文）
    let mut compacted_records = scope_records.clone();
    l1::compact_records(&mut compacted_records);

    // 达标检查：L1 后估算占用已低于触发阈值时跳过 L3，避免无谓的 LLM 摘要调用
    let post_l1_estimate = context::estimate_context_usage(history)
        + l1::estimate_tool_calls_size(&compacted_records) as u64;
    let l1_sufficient =
        post_l1_estimate <= (config.max_input_tokens as f64 * trigger::COMPRESS_THRESHOLD) as u64;

    // L3: 生成摘要（失败则降级为 L1+丢最旧）
    let summary_text = if l1_sufficient {
        None
    } else {
        l3::summarize(
            config,
            model,
            &l3::history_for_summary(history, &scope_records),
        )
        .await
    };

    // 持久化摘要（L3 成功才写；失败时清空避免旧摘要误导）
    if let Some(s) = &summary_text {
        let _ = db::upsert_summary(conversation_id, s);
    } else {
        let _ = db::delete_summary(conversation_id);
    }

    let mut turns = rebuild::rebuild(history, &compacted_records, summary_text.as_deref());
    turns.insert(0, rebuild::system_turn());

    Ok((
        turns,
        CompactInfo {
            compacted: true,
            reason: decision.reason.to_string(),
            has_summary: summary_text.is_some(),
        },
    ))
}
