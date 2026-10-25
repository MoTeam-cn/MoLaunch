//! 模组翻译：AI 响应解析、占位符保护
//!
//! 模型输出不可控，这里做三件事：剥围栏、按括号配对取第一个 JSON 对象、
//! 逐条取出译文；占位符校验统一走 `quality` 的扫描器。

use super::quality::extract_protected_tokens;
use super::types::{has_chinese, Loader};

/// 用户消息模板缺失时的兜底
const FALLBACK_TEMPLATE: &str = "请翻译 entries 中的条目，严格按此格式返回 JSON：{\"translations\":[{\"key\":\"原key\",\"translation\":\"简体中文译文\"}]}，key 与 entries 一一对应，不得遗漏或新增。\n\n{data}";

/// 译文是否合格：非空、含简体中文、占位符集合与源一致
pub fn validate_translation(source: &str, translation: &str) -> bool {
    let trimmed = translation.trim();
    !trimmed.is_empty()
        && has_chinese(trimmed)
        && extract_protected_tokens(source) == extract_protected_tokens(trimmed)
}

/// 括号配对的 JSON 扫描器：跳过字符串内的花括号与转义
#[derive(Default)]
struct JsonScanner {
    depth: i32,
    in_string: bool,
    escaped: bool,
}

impl JsonScanner {
    /// 处理一个字节；返回 true 表示对象闭合
    fn step(&mut self, byte: u8) -> bool {
        if self.in_string {
            if self.escaped {
                self.escaped = false;
            } else if byte == b'\\' {
                self.escaped = true;
            } else if byte == b'"' {
                self.in_string = false;
            }
            return false;
        }
        match byte {
            b'"' => {
                self.in_string = true;
                false
            }
            b'{' => {
                self.depth += 1;
                false
            }
            b'}' => {
                self.depth -= 1;
                self.depth == 0
            }
            _ => false,
        }
    }
}

/// 提取第一个完整 JSON 对象（模型可能在对象后追加解释文本）
pub fn extract_json_object(content: &str) -> Option<&str> {
    let start = content.find('{')?;
    let mut scanner = JsonScanner::default();
    for (index, byte) in content.as_bytes().iter().enumerate().skip(start) {
        if scanner.step(*byte) {
            return Some(&content[start..=index]);
        }
    }
    None
}

/// 剥离 markdown 代码围栏（```json … ```）
pub fn strip_json_fences(content: &str) -> &str {
    let trimmed = content.trim();
    let Some(body) = trimmed.strip_prefix("```") else {
        return trimmed;
    };
    let Some(body) = body.strip_suffix("```") else {
        return trimmed;
    };
    let body = body.trim();
    body.strip_prefix("json")
        .map(str::trim_start)
        .unwrap_or(body)
}

/// 解析 `{"translations":[{"key","translation"}]}`，丢弃空 key
pub fn parse_translations_response(content: &str) -> Result<Vec<(String, String)>, String> {
    let stripped = strip_json_fences(content);
    let json_text = extract_json_object(stripped).ok_or("AI 响应中未找到 JSON 对象")?;
    let value: serde_json::Value =
        serde_json::from_str(json_text).map_err(|e| format!("解析翻译 JSON 失败: {e}"))?;
    let items = value
        .get("translations")
        .and_then(serde_json::Value::as_array)
        .ok_or("AI 响应缺少 translations 数组")?;

    let entries = items
        .iter()
        .filter_map(|item| {
            let key = item.get("key").and_then(serde_json::Value::as_str)?;
            if key.is_empty() {
                return None;
            }
            let translation = item
                .get("translation")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            Some((key.to_string(), translation.to_string()))
        })
        .collect();
    Ok(entries)
}

/// 构造批量翻译用户消息：模板外置（编译期内嵌），以 `{data}` 注入条目数据
pub fn build_translation_user_prompt(
    loader: Loader,
    mod_ids: &[String],
    namespace: &str,
    entries: &[(String, String)],
) -> String {
    let payload = serde_json::json!({
        "loader": loader.as_str(),
        "modIds": mod_ids,
        "namespace": namespace,
        "entries": entries
            .iter()
            .map(|(key, source)| serde_json::json!({ "key": key, "source": source }))
            .collect::<Vec<_>>(),
    })
    .to_string();

    let template = crate::resources::read_resource("prompts/mod_translation_user.md")
        .unwrap_or_else(|_| FALLBACK_TEMPLATE.to_string());
    template.replace("{data}", &payload)
}

#[cfg(test)]
#[path = "prompt_test.rs"]
mod tests;
