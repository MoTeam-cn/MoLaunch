//! 模组翻译：保序 JSON 解析/渲染/指针写入（结构化资源写回用）

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

/// 递归下降解析器（按首字节分派）
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => Ok(JsonValue::String(self.parse_string()?)),
            Some(b't') => self.literal("true", JsonValue::Bool(true)),
            Some(b'f') => self.literal("false", JsonValue::Bool(false)),
            Some(b'n') => self.literal("null", JsonValue::Null),
            Some(_) => self.parse_number(),
            None => Err("JSON 文档意外结束".to_string()),
        }
    }

    fn literal(&mut self, text: &str, value: JsonValue) -> Result<JsonValue, String> {
        self.expect(text)?;
        Ok(value)
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1;
        let mut entries = Vec::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err("JSON 对象键必须是字符串".to_string());
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err("JSON 对象键缺少冒号".to_string());
            }
            self.pos += 1;
            entries.push((key, self.parse_value()?));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => return Err("JSON 对象缺少右花括号".to_string()),
            }
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1;
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err("JSON 数组缺少右方括号".to_string()),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.pos += 1;
        let mut out = String::new();
        loop {
            let Some(byte) = self.bytes.get(self.pos).copied() else {
                return Err("JSON 字符串未闭合".to_string());
            };
            self.pos += 1;
            match byte {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self
                        .bytes
                        .get(self.pos)
                        .copied()
                        .ok_or("JSON 转义序列未闭合")?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let Some(hex) = self.bytes.get(self.pos..self.pos + 4) else {
                                return Err("JSON \\u 转义被截断".to_string());
                            };
                            let code = u16::from_str_radix(std::str::from_utf8(hex).unwrap(), 16)
                                .unwrap_or(0xfffd);
                            self.pos += 4;
                            // 代理对：高代理（0xD800-0xDBFF）后紧跟 \u 低代理（0xDC00-0xDFFF）时合并
                            if (0xD800..=0xDBFF).contains(&code) {
                                let next = self.bytes.get(self.pos..self.pos + 6);
                                let low = next
                                    .filter(|n| n.starts_with(b"\\u"))
                                    .and_then(|n| {
                                        u16::from_str_radix(
                                            std::str::from_utf8(&n[2..6]).unwrap_or(""),
                                            16,
                                        )
                                        .ok()
                                    })
                                    .filter(|n| (0xDC00..=0xDFFF).contains(n));
                                if let Some(low) = low {
                                    let combined = 0x10000
                                        + ((code as u32 - 0xD800) << 10)
                                        + (low as u32 - 0xDC00);
                                    out.push(char::from_u32(combined).unwrap_or('\u{fffd}'));
                                    self.pos += 6;
                                    continue;
                                }
                                out.push('\u{fffd}');
                            } else {
                                out.push(char::from_u32(code as u32).unwrap_or('\u{fffd}'));
                            }
                        }
                        _ => return Err(format!("JSON 未知转义: \\{}", esc as char)),
                    }
                }
                byte if byte < 0x20 => return Err("JSON 字符串包含控制字符".to_string()),
                _ => {
                    let ch = std::str::from_utf8(&self.bytes[self.pos - 1..])
                        .unwrap()
                        .chars()
                        .next()
                        .unwrap();
                    out.push(ch);
                    self.pos += ch.len_utf8() - 1;
                }
            }
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        while self.pos < self.bytes.len()
            && matches!(
                self.bytes[self.pos],
                b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E'
            )
        {
            self.pos += 1;
        }
        if self.pos == start {
            return Err("JSON 包含非法 token".to_string());
        }
        std::str::from_utf8(&self.bytes[start..self.pos])
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map(JsonValue::Number)
            .ok_or_else(|| "JSON 数字非法".to_string())
    }

    fn expect(&mut self, literal: &str) -> Result<(), String> {
        if !self.bytes[self.pos..].starts_with(literal.as_bytes()) {
            return Err(format!("JSON 期望字面量 {literal}"));
        }
        self.pos += literal.len();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_preserves_key_order() {
        let root = JsonValue::parse(r#"{"z":"1","a":"2","m":"3"}"#).unwrap();
        let rendered = root.render_pretty();
        let z = rendered.find("\"z\"").unwrap();
        let a = rendered.find("\"a\"").unwrap();
        let m = rendered.find("\"m\"").unwrap();
        assert!(z < a && a < m, "键序必须保留: {rendered}");
    }

    #[test]
    fn set_pointer_writes_nested_value() {
        let mut root = JsonValue::parse(r#"{"a":{"b":[{"c":"old"}]}}"#).unwrap();
        root.set_pointer("/a/b/0/c", "new".to_string()).unwrap();
        assert!(root.render_pretty().contains("\"c\": \"new\""));
    }

    #[test]
    fn render_integers_without_decimal() {
        let root = JsonValue::parse(r#"{"n":10,"f":1.5}"#).unwrap();
        let rendered = root.render_pretty();
        assert!(rendered.contains("\"n\": 10") && rendered.contains("\"f\": 1.5"));
    }
}
