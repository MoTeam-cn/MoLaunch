//! L1 微压缩纯函数测试（同目录测试文件，`cargo test l1` 运行）

use crate::commands::experimental::types::ToolCallRecord;

use super::l1::{compact_records, estimate_tool_calls_size, truncate_lines, truncate_text};

#[test]
fn truncate_short_text_unchanged() {
    let text = "短文本内容";
    assert_eq!(truncate_text(text, 10, 10), text);
}

#[test]
fn truncate_long_text_keeps_head_and_tail() {
    let text = "a".repeat(100);
    let out = truncate_text(&text, 10, 5);
    assert!(out.starts_with("aaaaaaaaaa"));
    assert!(out.ends_with("aaaaa"));
    assert!(out.contains("截断"));
    assert!(out.len() < text.len());
}

#[test]
fn truncate_boundary_no_overlap() {
    // 恰好等于 head+tail 时不截断
    let text = "b".repeat(15);
    assert_eq!(truncate_text(&text, 10, 5), text);
    // 超过 1 个字符才截断
    let over = "b".repeat(16);
    assert_ne!(truncate_text(&over, 10, 5), over);
}

#[test]
fn compact_records_truncates_only_oversized() {
    let mut records = vec![
        ToolCallRecord {
            message_id: 1,
            seq: 0,
            name: "big".to_string(),
            arguments: String::new(),
            output: Some("x".repeat(6_000)),
            pre_content: None,
        },
        ToolCallRecord {
            message_id: 1,
            seq: 1,
            name: "small".to_string(),
            arguments: String::new(),
            output: Some("short".to_string()),
            pre_content: None,
        },
    ];
    let saved = compact_records(&mut records);
    assert!(saved > 0);
    assert!(records[0].output.as_deref().unwrap().len() < 6_000);
    assert_eq!(records[1].output.as_deref().unwrap(), "short");
}

#[test]
fn estimate_tool_calls_size_sums_outputs_and_arguments() {
    let records = vec![
        ToolCallRecord {
            message_id: 1,
            seq: 0,
            name: "a".to_string(),
            arguments: "{}".to_string(),
            output: Some("hello".to_string()),
            pre_content: None,
        },
        ToolCallRecord {
            message_id: 1,
            seq: 1,
            name: "b".to_string(),
            arguments: "[]".to_string(),
            output: None,
            pre_content: None,
        },
    ];
    // 5（hello）+ 2（{}）+ 2（[]）+ 0（None）= 9
    assert_eq!(estimate_tool_calls_size(&records), 9);
}

#[test]
fn truncate_lines_short_unchanged() {
    let text = "a\nb\nc";
    assert_eq!(truncate_lines(text, 2, 2), text);
}

#[test]
fn truncate_lines_keeps_head_tail_with_marker() {
    let mut text = String::new();
    for i in 0..2100 {
        text.push_str(&format!("line{}\n", i));
    }
    let out = truncate_lines(&text, 20, 20);
    assert!(out.starts_with("line0\n"));
    assert!(out.contains("截断 2060 行"));
    assert!(out.ends_with("line2099\n"));
    assert!(out.len() < text.len());
}

#[test]
fn compact_records_compacts_big_json_top_keys() {
    let mut root = serde_json::Map::new();
    for i in 0..1001 {
        root.insert(
            format!("k{}", i),
            serde_json::json!({"id": i, "nested": [1, 2, 3]}),
        );
    }
    let json = serde_json::to_string(&serde_json::json!({"root": root})).unwrap();
    let mut records = vec![ToolCallRecord {
        message_id: 1,
        seq: 0,
        name: "read".to_string(),
        arguments: String::new(),
        output: Some(json),
        pre_content: None,
    }];
    let saved = compact_records(&mut records);
    assert!(saved > 0);
    let out = records[0].output.as_deref().unwrap();
    // 仅保留顶层 key，嵌套的 k0..k1000 被占位
    assert!(out.contains("\"root\": {...}"));
    assert!(!out.contains("\"k0\""));
}

#[test]
fn compact_records_falls_back_for_small_node_json() {
    // 节点数少但文本超长：退化为字符头尾截断
    let json = format!("{{\"a\": \"{}\"}}", "x".repeat(6_000));
    let mut records = vec![ToolCallRecord {
        message_id: 1,
        seq: 0,
        name: "read".to_string(),
        arguments: String::new(),
        output: Some(json),
        pre_content: None,
    }];
    compact_records(&mut records);
    let out = records[0].output.as_deref().unwrap();
    assert!(out.contains("截断"));
    assert!(out.starts_with("{\"a\": \"xxx"));
}
