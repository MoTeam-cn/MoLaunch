//! 保序 JSON 解析器（规格 §2 语言源识别）
//!
//! 按首字节分派：对象 / 数组 / 字符串 / 数字 / 字面量。
//! 全部读取走带边界检查的 `Reader`，非法输入返回中文错误，不 panic。

use super::value::JsonValue;

/// 解析完整文档，拒绝尾随内容
pub(super) fn parse_document(input: &str) -> Result<JsonValue, String> {
    let mut reader = Reader::new(input);
    let value = reader.value()?;
    reader.skip_whitespace();
    if !reader.at_end() {
        return Err("JSON 文档存在尾随内容".to_string());
    }
    Ok(value)
}

/// 字节游标
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), String> {
        if self.next_byte() == Some(expected) {
            return Ok(());
        }
        Err(format!("JSON 期望字符 {}", expected as char))
    }

    fn value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(JsonValue::String(self.string()?)),
            Some(b't') => self.literal("true", JsonValue::Bool(true)),
            Some(b'f') => self.literal("false", JsonValue::Bool(false)),
            Some(b'n') => self.literal("null", JsonValue::Null),
            Some(_) => self.number(),
            None => Err("JSON 文档意外结束".to_string()),
        }
    }

    fn literal(&mut self, text: &str, value: JsonValue) -> Result<JsonValue, String> {
        if !self.bytes[self.pos..].starts_with(text.as_bytes()) {
            return Err(format!("JSON 期望字面量 {text}"));
        }
        self.pos += text.len();
        Ok(value)
    }

    fn object(&mut self) -> Result<JsonValue, String> {
        self.expect_byte(b'{')?;
        let mut entries: Vec<(String, JsonValue)> = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err("JSON 对象键必须是字符串".to_string());
            }
            let key = self.string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            let value = self.value()?;
            entries.push((key, value));
            self.skip_whitespace();
            match self.next_byte() {
                Some(b',') => continue,
                Some(b'}') => return Ok(JsonValue::Object(entries)),
                _ => return Err("JSON 对象缺少右花括号".to_string()),
            }
        }
    }

    fn array(&mut self) -> Result<JsonValue, String> {
        self.expect_byte(b'[')?;
        let mut items: Vec<JsonValue> = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_whitespace();
            match self.next_byte() {
                Some(b',') => continue,
                Some(b']') => return Ok(JsonValue::Array(items)),
                _ => return Err("JSON 数组缺少右方括号".to_string()),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect_byte(b'"')?;
        let mut out = String::new();
        loop {
            let byte = self
                .next_byte()
                .ok_or_else(|| "JSON 字符串未闭合".to_string())?;
            match byte {
                b'"' => return Ok(out),
                b'\\' => self.escape(&mut out)?,
                0x00..=0x1f => return Err("JSON 字符串包含控制字符".to_string()),
                0x20..=0x7f => out.push(byte as char),
                _ => {
                    // 多字节字符：回退后按字符边界读取
                    self.pos -= 1;
                    let rest = std::str::from_utf8(&self.bytes[self.pos..])
                        .map_err(|_| "JSON 字符串不是合法 UTF-8".to_string())?;
                    let ch = rest
                        .chars()
                        .next()
                        .ok_or_else(|| "JSON 字符串未闭合".to_string())?;
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
    }

    fn escape(&mut self, out: &mut String) -> Result<(), String> {
        let marker = self
            .next_byte()
            .ok_or_else(|| "JSON 转义序列未闭合".to_string())?;
        match marker {
            b'"' => out.push('"'),
            b'\\' => out.push('\\'),
            b'/' => out.push('/'),
            b'b' => out.push('\u{0008}'),
            b'f' => out.push('\u{000c}'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'u' => {
                let high = self.hex4()?;
                match surrogate_pair(high, self.peek_low_surrogate()) {
                    Some((ch, skip)) => {
                        out.push(ch);
                        self.pos += skip;
                    }
                    None => out.push(char::from_u32(high as u32).unwrap_or('\u{fffd}')),
                }
            }
            other => return Err(format!("JSON 未知转义: \\{}", other as char)),
        }
        Ok(())
    }

    /// 读 4 位十六进制
    fn hex4(&mut self) -> Result<u16, String> {
        let slice = self
            .bytes
            .get(self.pos..self.pos + 4)
            .ok_or_else(|| "JSON \\u 转义被截断".to_string())?;
        let text = std::str::from_utf8(slice).map_err(|_| "JSON \\u 转义非法".to_string())?;
        let code = u16::from_str_radix(text, 16).map_err(|_| "JSON \\u 转义非法".to_string())?;
        self.pos += 4;
        Ok(code)
    }

    /// 紧随其后的低代理项（`\\uDC00` ~ `\\uDFFF`）偏移
    fn peek_low_surrogate(&self) -> Option<u16> {
        let slice = self.bytes.get(self.pos..self.pos + 6)?;
        if !slice.starts_with(b"\\u") {
            return None;
        }
        let text = std::str::from_utf8(&slice[2..6]).ok()?;
        let code = u16::from_str_radix(text, 16).ok()?;
        (0xDC00..=0xDFFF).contains(&code).then_some(code)
    }

    fn number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        while matches!(
            self.peek(),
            Some(b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')
        ) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err("JSON 包含非法 token".to_string());
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| "JSON 数字非法".to_string())?;
        text.parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| "JSON 数字非法".to_string())
    }
}

/// 高代理 + 低代理 → 合并字符（返回合并结果与需跳过的字节数）
fn surrogate_pair(high: u16, low: Option<u16>) -> Option<(char, usize)> {
    if !(0xD800..=0xDBFF).contains(&high) {
        return None;
    }
    let low = low?;
    let combined = 0x10000 + ((high as u32 - 0xD800) << 10) + (low as u32 - 0xDC00);
    char::from_u32(combined).map(|ch| (ch, 6))
}
