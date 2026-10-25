//! 保序 JSON 值：对象用 Vec 保持键序，避免排序打乱原文件结构

use super::parser;
use super::writer;

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
    /// 解析完整 JSON 文档
    pub fn parse(input: &str) -> Result<JsonValue, String> {
        parser::parse_document(input)
    }

    /// 按 JSON Pointer 写入字符串（`~0`/`~1` 反转义，路径缺失报错）
    pub fn set_pointer(&mut self, pointer: &str, text: String) -> Result<(), String> {
        if !pointer.starts_with('/') {
            return Err(format!("JSON Pointer 必须以 / 开头: {pointer}"));
        }
        if pointer == "/" {
            return match self {
                JsonValue::String(current) => {
                    *current = text;
                    Ok(())
                }
                _ => Err("JSON Pointer / 指向非字符串根节点".to_string()),
            };
        }

        let segments: Vec<String> = pointer
            .split('/')
            .skip(1)
            .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
            .collect();

        let mut cursor = self;
        for (position, segment) in segments.iter().enumerate() {
            let last = position + 1 == segments.len();
            cursor = match cursor {
                JsonValue::Object(entries) => {
                    let index = entries
                        .iter()
                        .position(|(key, _)| key == segment)
                        .ok_or_else(|| format!("JSON Pointer 段不存在: {segment}"))?;
                    if last {
                        entries[index].1 = JsonValue::String(text);
                        return Ok(());
                    }
                    &mut entries[index].1
                }
                JsonValue::Array(items) => {
                    let index: usize = segment
                        .parse()
                        .map_err(|_| format!("JSON Pointer 段不是数组索引: {segment}"))?;
                    let item = items
                        .get_mut(index)
                        .ok_or_else(|| format!("JSON Pointer 索引越界: {segment}"))?;
                    if last {
                        *item = JsonValue::String(text);
                        return Ok(());
                    }
                    item
                }
                _ => return Err(format!("JSON Pointer 穿越标量节点: {segment}")),
            };
        }
        Err(format!("JSON Pointer 未解析到字符串: {pointer}"))
    }

    /// 渲染为两空格缩进的 JSON 文本
    pub fn render_pretty(&self) -> String {
        writer::render(self)
    }
}
