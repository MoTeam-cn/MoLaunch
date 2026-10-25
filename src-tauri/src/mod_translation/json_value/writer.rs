//! 保序 JSON 渲染（两空格缩进，字符串转义）

use super::value::JsonValue;

/// 渲染整篇文档（末尾补换行）
pub(super) fn render(value: &JsonValue) -> String {
    let mut out = String::new();
    write_value(value, &mut out, 0);
    out.push('\n');
    out
}

fn write_value(value: &JsonValue, out: &mut String, depth: usize) {
    match value {
        JsonValue::Object(entries) => write_object(entries, out, depth),
        JsonValue::Array(items) => write_array(items, out, depth),
        JsonValue::String(text) => out.push_str(&quote(text)),
        JsonValue::Number(number) => out.push_str(&format_number(*number)),
        JsonValue::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        JsonValue::Null => out.push_str("null"),
    }
}

fn write_object(entries: &[(String, JsonValue)], out: &mut String, depth: usize) {
    if entries.is_empty() {
        out.push_str("{}");
        return;
    }
    out.push_str("{\n");
    for (index, (key, value)) in entries.iter().enumerate() {
        indent(out, depth + 1);
        out.push_str(&quote(key));
        out.push_str(": ");
        write_value(value, out, depth + 1);
        if index + 1 < entries.len() {
            out.push(',');
        }
        out.push('\n');
    }
    indent(out, depth);
    out.push('}');
}

fn write_array(items: &[JsonValue], out: &mut String, depth: usize) {
    if items.is_empty() {
        out.push_str("[]");
        return;
    }
    out.push_str("[\n");
    for (index, value) in items.iter().enumerate() {
        indent(out, depth + 1);
        write_value(value, out, depth + 1);
        if index + 1 < items.len() {
            out.push(',');
        }
        out.push('\n');
    }
    indent(out, depth);
    out.push(']');
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// 整数原样输出，其余按最短往返表示
fn format_number(number: f64) -> String {
    if number.fract() == 0.0 && number.abs() < 9_007_199_254_740_992.0 {
        return format!("{}", number as i64);
    }
    number.to_string()
}

/// 字符串转义：引号 / 反斜杠 / 控制字符，非 ASCII 原样输出
fn quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}
