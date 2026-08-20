//! 递归下降 JSON 解析器（按首字节分派）

use super::value::JsonValue;

/// 递归下降解析器（按首字节分派）
pub(super) struct Parser<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) pos: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn skip_ws(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    pub(super) fn parse_value(&mut self) -> Result<JsonValue, String> {
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
