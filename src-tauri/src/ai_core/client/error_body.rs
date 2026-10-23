//! AI 服务错误响应解析
//!
//! 各家 OpenAI 兼容服务出错时的响应体并不统一：`{"error": {"message": ...}}`、
//! `{"error": "..."}`、`{"error": [{"message": ...}]}`、`{"errors": [...]}`、
//! `{"message": ...}` 都出现过。这里统一抽出可读原因，让所有调用方直接把服务端
//! 给的错误展示给用户，而不是把整段 JSON 或只把状态码抛上去。

use serde_json::Value;

/// 错误文本最大字符数（超出截断，避免把整段 HTML 错误页塞进提示）
const MAX_CHARS: usize = 300;

/// 常见错误字段名（按优先级）
const MESSAGE_KEYS: [&str; 7] = [
    "message",
    "msg",
    "detail",
    "error_description",
    "error_msg",
    "reason",
    "title",
];

/// 需要递归下钻的容器字段名
const NESTED_KEYS: [&str; 5] = ["error", "errors", "details", "data", "body"];

fn push_unique(out: &mut Vec<String>, raw: &str) {
    let text = raw.trim();
    if text.is_empty() {
        return;
    }
    // 纯符号/纯数字占位串（如 "0"、"-"）没有信息量，跳过
    if text.chars().all(|c| !c.is_alphanumeric()) {
        return;
    }
    if out.iter().any(|s| s == text) {
        return;
    }
    out.push(text.to_string());
}

/// 递归收集响应体里的错误文本
fn collect(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => push_unique(out, s),
        Value::Array(items) => {
            for item in items {
                collect(item, out);
            }
        }
        Value::Object(map) => {
            for key in MESSAGE_KEYS {
                if let Some(Value::String(s)) = map.get(key) {
                    push_unique(out, s);
                }
            }
            for key in NESTED_KEYS {
                if let Some(nested) = map.get(key) {
                    collect(nested, out);
                }
            }
        }
        _ => {}
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max).collect();
    format!("{}…", cut)
}

/// 从错误响应体提取可读原因（多段错误用「；」连接；非 JSON 时回退为压缩空白后的原文）
pub(crate) fn extract_error_message(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        collect(&value, &mut parts);
    }

    let message = if parts.is_empty() {
        trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        parts.join("；")
    };
    if message.is_empty() {
        return None;
    }
    Some(truncate(&message, MAX_CHARS))
}

/// 统一的 HTTP 错误文案：`AI 服务返回 HTTP 400: <服务端原因>`
///
/// 所有调用方（对话、日志分析、模型列表、工具循环）共用同一文案，
/// 前端把 `e.to_string()` 直接展示即可看到服务端给出的具体原因。
pub(crate) fn http_error_message(status: u16, body: &str) -> String {
    match extract_error_message(body) {
        Some(msg) => format!("AI 服务返回 HTTP {}: {}", status, msg),
        None => format!("AI 服务返回 HTTP {}（服务端未返回错误详情）", status),
    }
}

#[cfg(test)]
#[path = "error_body_test.rs"]
mod error_body_test;
