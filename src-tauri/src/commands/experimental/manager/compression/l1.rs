//! L1 微压缩：工具输出截断（不调用 LLM，纯文本处理）
//!
//! 按内容形态选择策略：JSON 结构体 > 1000 节点时保留顶层 Key；
//! 代码/日志 > 2000 行时保留首尾行；其余超长文本按字符头尾保留。
//! 同时汇总统计工具输出总体积，供触发判定器评估"巨型工具输出"指标。

use crate::commands::experimental::types::ToolCallRecord;

/// 单条工具输出截断阈值（字符）
const L1_TEXT_THRESHOLD: usize = 5_000;
/// 截断后保留头部字符数
const L1_KEEP_HEAD: usize = 2_500;
/// 截断后保留尾部字符数
const L1_KEEP_TAIL: usize = 2_000;
/// 代码/日志行级截断阈值（行数）
const L1_LINE_THRESHOLD: usize = 2_000;
/// 行级截断保留头部行数
const L1_KEEP_HEAD_LINES: usize = 20;
/// 行级截断保留尾部行数
const L1_KEEP_TAIL_LINES: usize = 20;
/// JSON 结构节点截断阈值（超过后仅保留顶层 Key）
const L1_JSON_NODE_THRESHOLD: usize = 1_000;

/// 工具输出总量（字符，供触发判定）
pub fn estimate_tool_calls_size(records: &[ToolCallRecord]) -> usize {
    records
        .iter()
        .map(|r| r.output.as_deref().unwrap_or("").len() + r.arguments.len())
        .sum()
}

/// 截断单条文本：超过阈值时保留头尾，中间省略
pub fn truncate_text(text: &str, keep_head: usize, keep_tail: usize) -> String {
    if text.len() <= keep_head + keep_tail {
        return text.to_string();
    }
    let mut head = String::new();
    let mut tail = String::new();
    for c in text.chars() {
        if head.len() < keep_head {
            head.push(c);
        } else {
            break;
        }
    }
    for c in text.chars().rev() {
        if tail.len() < keep_tail {
            tail.insert(0, c);
        } else {
            break;
        }
    }
    format!(
        "{}…[截断 {} 字符]…{}",
        head,
        text.len() - head.len() - tail.len(),
        tail
    )
}

/// 截断代码/日志：超过阈值行数时保留首尾行，中间替换为省略标记
pub fn truncate_lines(text: &str, keep_head: usize, keep_tail: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= keep_head + keep_tail {
        return text.to_string();
    }
    let mut out = String::new();
    for line in &lines[..keep_head] {
        out.push_str(line);
        out.push('\n');
    }
    let skipped = lines.len() - keep_head - keep_tail;
    out.push_str(&format!("[... 截断 {} 行 ...]\n", skipped));
    for line in &lines[lines.len() - keep_tail..] {
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// 递归统计 JSON 节点数（对象/数组及其子节点均计 1）
fn count_json_nodes(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => 1 + map.values().map(count_json_nodes).sum::<usize>(),
        serde_json::Value::Array(arr) => 1 + arr.iter().map(count_json_nodes).sum::<usize>(),
        _ => 1,
    }
}

/// JSON 顶层 Key 精简：仅保留顶层键，嵌套内容替换为类型占位
///
/// 节点数未超过阈值或无法解析时返回 None（由调用方退化为其他截断策略）。
fn compact_json_top_keys(text: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    if count_json_nodes(&value) <= L1_JSON_NODE_THRESHOLD {
        return None;
    }
    Some(match value {
        serde_json::Value::Object(map) => {
            let mut out = String::from("{");
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                let placeholder = match v {
                    serde_json::Value::Object(_) => "{...}".to_string(),
                    serde_json::Value::Array(a) => format!("[... {} 项 ...]", a.len()),
                    serde_json::Value::String(s) => {
                        if s.chars().count() <= 120 {
                            format!("\"{}\"", s)
                        } else {
                            format!("\"{}\"", crate::utils::format::truncate_chars(s, 60))
                        }
                    }
                    serde_json::Value::Null => "null".to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                };
                out.push_str(&format!("\"{}\": {}", k, placeholder));
            }
            out.push('}');
            out
        }
        serde_json::Value::Array(a) => format!("[... {} 项 ...]", a.len()),
        _ => return None,
    })
}

/// 按内容形态选择压缩策略（JSON 顶层精简 / 行级截断 / 字符头尾保留）
fn compress_output(out: &str) -> String {
    if out.len() <= L1_TEXT_THRESHOLD {
        return out.to_string();
    }
    if let Some(compacted) = compact_json_top_keys(out) {
        return compacted;
    }
    if out.lines().count() > L1_LINE_THRESHOLD {
        return truncate_lines(out, L1_KEEP_HEAD_LINES, L1_KEEP_TAIL_LINES);
    }
    truncate_text(out, L1_KEEP_HEAD, L1_KEEP_TAIL)
}

/// 对工具调用记录做 L1 微压缩：巨型输出按内容形态截断
///
/// 返回压缩前后字符数变化；无变化说明无需压缩。
pub fn compact_records(records: &mut [ToolCallRecord]) -> usize {
    let mut saved = 0usize;
    for r in records.iter_mut() {
        if let Some(out) = r.output.take() {
            let after = compress_output(&out);
            saved += out.len().saturating_sub(after.len());
            r.output = Some(after);
        }
    }
    saved
}
