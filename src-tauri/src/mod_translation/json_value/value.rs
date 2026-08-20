//! 保序 JSON 值：对象用 Vec 保持键序，避免 serde_json 排序打乱原文件结构

use super::parser::Parser;

/// 保序 JSON 值：对象用 Vec 保持键序，避免 serde_json 排序打乱原文件结构
#[derive(Debug, Clone)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

impl JsonValue {
    /// 解析完整 JSON 文档（拒绝尾随内容）
    pub fn parse(input: &str) -> Result<JsonValue, String> {
        let mut parser = Parser {
            bytes: input.as_bytes(),
            pos: 0,
        };
        let value = parser.parse_value()?;
        parser.skip_ws();
        if parser.pos != input.len() {
            return Err("JSON 文档存在尾随内容".to_string());
        }
        Ok(value)
    }

    /// 按 JSON Pointer 定位并写入字符串（~0/~1 反转义，路径缺失报错）
    pub fn set_pointer(&mut self, pointer: &str, value: String) -> Result<(), String> {
        if !pointer.starts_with('/') {
            return Err(format!("JSON Pointer 必须以 / 开头: {pointer}"));
        }
        if pointer == "/" {
            if let JsonValue::String(s) = self {
                *s = value;
                return Ok(());
            }
            return Err("JSON Pointer / 指向非字符串根节点".to_string());
        }
        let segments: Vec<String> = pointer
            .split('/')
            .skip(1)
            .map(|seg| seg.replace("~1", "/").replace("~0", "~"))
            .collect();
        let mut cursor = self;
        for (idx, segment) in segments.iter().enumerate() {
            let last = idx + 1 == segments.len();
            match cursor {
                JsonValue::Object(entries) => {
                    let Some(pos) = entries.iter().position(|(k, _)| k == segment) else {
                        return Err(format!("JSON Pointer 段不存在: {segment}"));
                    };
                    if last {
                        entries[pos].1 = JsonValue::String(value);
                        return Ok(());
                    }
                    cursor = &mut entries[pos].1;
                }
                JsonValue::Array(items) => {
                    let index: usize = segment
                        .parse()
                        .map_err(|_| format!("JSON Pointer 段不是数组索引: {segment}"))?;
                    let Some(item) = items.get_mut(index) else {
                        return Err(format!("JSON Pointer 索引越界: {segment}"));
                    };
                    if last {
                        *item = JsonValue::String(value);
                        return Ok(());
                    }
                    cursor = item;
                }
                _ => return Err(format!("JSON Pointer 穿越标量节点: {segment}")),
            }
        }
        Err(format!("JSON Pointer 未解析到字符串: {pointer}"))
    }

    pub fn render_pretty(&self) -> String {
        let mut out = String::new();
        self.write_pretty(&mut out, 0);
        out.push('\n');
        out
    }

    fn write_pretty(&self, out: &mut String, indent: usize) {
        match self {
            JsonValue::Object(entries) => {
                if entries.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push_str("{\n");
                for (i, (key, value)) in entries.iter().enumerate() {
                    out.push_str(&"  ".repeat(indent + 1));
                    out.push_str(&escape_string(key));
                    out.push_str(": ");
                    value.write_pretty(out, indent + 1);
                    if i + 1 < entries.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str(&"  ".repeat(indent));
                out.push('}');
            }
            JsonValue::Array(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push_str("[\n");
                for (i, value) in items.iter().enumerate() {
                    out.push_str(&"  ".repeat(indent + 1));
                    value.write_pretty(out, indent + 1);
                    if i + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str(&"  ".repeat(indent));
                out.push(']');
            }
            JsonValue::String(s) => out.push_str(&escape_string(s)),
            JsonValue::Number(n) if n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 => {
                out.push_str(&format!("{}", *n as i64));
            }
            JsonValue::Number(n) => out.push_str(&n.to_string()),
            JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            JsonValue::Null => out.push_str("null"),
        }
    }
}

/// 字符串转义：引号/反斜杠/控制字符，非 ASCII 原样输出
fn escape_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
