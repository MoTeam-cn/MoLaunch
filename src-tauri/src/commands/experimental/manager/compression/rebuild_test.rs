//! 上下文重塑器测试（同目录测试文件，`cargo test rebuild` 运行）

use super::rebuild::{rebuild, RECENT_KEEP};
use super::test_support::{msg, tool};

#[test]
fn rebuild_injects_summary_and_boundary() {
    let history = vec![msg(1, "user", "a"), msg(2, "assistant", "b")];
    let turns = rebuild(&history, &[], Some("汇总内容"));
    assert_eq!(turns.len(), 4);
    assert!(turns[0].content.as_deref().unwrap().contains("压缩摘要"));
    assert!(turns[0].content.as_deref().unwrap().contains("汇总内容"));
    assert!(turns[1]
        .content
        .as_deref()
        .unwrap()
        .contains("COMPACT_BOUNDARY"));
    assert_eq!(turns[2].role, "user");
    assert_eq!(turns[3].role, "assistant");
}

#[test]
fn rebuild_without_summary_starts_at_boundary() {
    let history = vec![msg(1, "user", "a")];
    let turns = rebuild(&history, &[], None);
    assert_eq!(turns.len(), 2);
    // 无摘要时第一帧即为边界，而非摘要
    assert!(turns[0]
        .content
        .as_deref()
        .unwrap()
        .contains("COMPACT_BOUNDARY"));
}

#[test]
fn rebuild_keeps_only_recent() {
    let history: Vec<_> = (0..20)
        .map(|i| {
            msg(
                i as i64,
                if i % 2 == 0 { "user" } else { "assistant" },
                &format!("msg{}", i),
            )
        })
        .collect();
    let turns = rebuild(&history, &[], Some("s"));
    // 摘要 + 边界 + RECENT_KEEP 条
    assert_eq!(turns.len(), 2 + RECENT_KEEP);
    // 最旧的 5 条被丢弃
    assert!(!turns.iter().any(|t| t.content.as_deref() == Some("msg0")));
    // 保留窗口从 msg5 起
    assert_eq!(turns[2].content.as_deref(), Some("msg5"));
    assert_eq!(turns.last().unwrap().content.as_deref(), Some("msg19"));
}

#[test]
fn rebuild_injects_tool_blocks_into_assistant() {
    let history = vec![
        msg(1, "user", "hi"),
        msg(2, "assistant", "我来查一下"),
        msg(3, "user", "ok"),
    ];
    let records = vec![tool(2, "read_file", Some("文件内容"))];
    let turns = rebuild(&history, &records, None);
    // 边界 + 3 条消息
    assert_eq!(turns.len(), 4);
    let assistant = turns[2].content.as_deref().unwrap();
    assert!(assistant.contains("【工具调用：read_file】"));
    assert!(assistant.contains("文件内容"));
    // 用户消息不被注入工具块
    assert!(!turns[1].content.as_deref().unwrap().contains("【工具调用"));
}
