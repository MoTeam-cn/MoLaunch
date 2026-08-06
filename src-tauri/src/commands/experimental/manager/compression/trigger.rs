//! 压缩触发判定器（多指标 + 防抖）
//!
//! Token 占用（真实 usage 或退化估算）、消息条数、巨型工具输出
//! 任一达标即触发；同一会话 30s 内防抖，避免连续消息反复压缩。

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use once_cell::sync::Lazy;

use super::super::context;
use super::l1::estimate_tool_calls_size;
use crate::commands::experimental::types::{MessageItem, ToolCallRecord};

/// 上下文窗口使用率上限（超过此比例触发压缩）
pub(super) const COMPRESS_THRESHOLD: f64 = 0.8;
/// 消息条数触发阈值
pub(super) const MESSAGE_COUNT_THRESHOLD: usize = 50;
/// 巨型工具输出触发阈值（字符数）
pub(super) const TOOL_OUTPUT_THRESHOLD: usize = 20_000;
/// 同会话压缩冷却时间（秒）
pub(super) const COOLDOWN_SECS: u64 = 30;

/// 会话最近压缩时间戳（防抖）
static LAST_COMPACT: Lazy<Mutex<HashMap<i64, Instant>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 触发判定结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TriggerDecision {
    /// 是否需要压缩
    pub should_compact: bool,
    /// 是否处于冷却期（冷却期内不重复压缩）
    pub in_cooldown: bool,
    /// 触发原因描述
    pub reason: &'static str,
}

/// 评估当前会话是否需要压缩
pub(super) fn evaluate(
    conversation_id: i64,
    history: &[MessageItem],
    tool_records: &[ToolCallRecord],
    max_input_tokens: u32,
) -> TriggerDecision {
    // 冷却期内跳过（避免用户连续发送时反复压缩）
    let now = Instant::now();
    let last = LAST_COMPACT.lock().unwrap().get(&conversation_id).copied();
    if let Some(t) = last {
        if now.duration_since(t).as_secs() < COOLDOWN_SECS {
            return TriggerDecision {
                should_compact: false,
                in_cooldown: true,
                reason: "冷却期",
            };
        }
    }

    // 指标 1：Token 占用（复用 context 的估算：真实 usage 优先，无 usage 时退化为字符估算）
    let total = context::estimate_context_usage(history);
    if total > (max_input_tokens as f64 * COMPRESS_THRESHOLD) as u64 {
        return TriggerDecision {
            should_compact: true,
            in_cooldown: false,
            reason: "Token 占用过高",
        };
    }

    // 指标 2：消息条数
    if history.len() >= MESSAGE_COUNT_THRESHOLD {
        return TriggerDecision {
            should_compact: true,
            in_cooldown: false,
            reason: "消息条数过多",
        };
    }

    // 指标 3：巨型工具输出
    if estimate_tool_calls_size(tool_records) > TOOL_OUTPUT_THRESHOLD {
        return TriggerDecision {
            should_compact: true,
            in_cooldown: false,
            reason: "工具输出过大",
        };
    }

    TriggerDecision {
        should_compact: false,
        in_cooldown: false,
        reason: "",
    }
}

/// 记录本次压缩时间（压缩成功后调用，进入冷却）
pub(super) fn record_compaction(conversation_id: i64) {
    LAST_COMPACT
        .lock()
        .unwrap()
        .insert(conversation_id, Instant::now());
}

/// 清理会话的防抖记录（会话删除时调用，避免内存泄漏）
pub(crate) fn clear_cooldown(conversation_id: i64) {
    LAST_COMPACT.lock().unwrap().remove(&conversation_id);
}
