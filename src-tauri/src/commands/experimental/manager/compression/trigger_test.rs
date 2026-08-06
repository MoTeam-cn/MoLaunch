//! 触发判定器测试（同目录测试文件，`cargo test trigger` 运行）

use super::test_support::{msg, tool};
use super::trigger::{evaluate, record_compaction, TriggerDecision};

/// 每个测试使用独立的会话 id，避免全局防抖表互相干扰
fn small_history() -> Vec<crate::commands::experimental::types::MessageItem> {
    vec![msg(1, "user", "你好"), msg(2, "assistant", "我在")]
}

#[test]
fn small_history_no_trigger() {
    let history = small_history();
    let d = evaluate(1001, &history, &[], 10_000);
    assert!(!d.should_compact);
    assert!(!d.in_cooldown);
    assert_eq!(d.reason, "");
}

#[test]
fn message_count_triggers() {
    let history: Vec<_> = (0..50).map(|i| msg(i as i64, "user", "hi")).collect();
    let d = evaluate(1002, &history, &[], 10_000);
    assert!(d.should_compact);
    assert!(!d.in_cooldown);
    assert_eq!(d.reason, "消息条数过多");
}

#[test]
fn token_usage_triggers() {
    // 1000 个 CJK 字符 ≈ 1000 token，超过 1000 * 0.8 = 800 的触发线
    let history = vec![msg(1, "user", &"长".repeat(1_000))];
    let d = evaluate(1003, &history, &[], 1_000);
    assert!(d.should_compact);
    assert_eq!(d.reason, "Token 占用过高");
}

#[test]
fn token_usage_prefers_real_prompt_tokens() {
    // 有真实 usage 时按 usage 判定（复用 context::estimate_context_usage 的真实分支）
    let mut history = small_history();
    let last = &mut history[1];
    last.prompt_tokens = Some(900);
    let d = evaluate(1004, &history, &[], 1_000);
    assert!(d.should_compact);
    assert_eq!(d.reason, "Token 占用过高");
}

#[test]
fn huge_tool_output_triggers() {
    let history = small_history();
    let records = vec![tool(2, "read", Some(&"x".repeat(25_000)))];
    let d = evaluate(1005, &history, &records, 10_000);
    assert!(d.should_compact);
    assert!(!d.in_cooldown);
    assert_eq!(d.reason, "工具输出过大");
}

#[test]
fn cooldown_blocks_repeat() {
    let conversation_id = 1006;
    let history = small_history();
    record_compaction(conversation_id);
    let d = evaluate(conversation_id, &history, &[], 10_000);
    assert!(!d.should_compact);
    assert!(d.in_cooldown);
    assert_eq!(d.reason, "冷却期");
}

#[test]
fn decision_fields_consistent() {
    let history = small_history();
    let d: TriggerDecision = evaluate(1007, &history, &[], 10_000);
    assert!(!d.should_compact);
    assert!(!d.in_cooldown);
}
