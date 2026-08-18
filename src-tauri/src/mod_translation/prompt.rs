//! 模组翻译：AI 响应解析、占位符保护

use super::quality::extract_protected_tokens;
use super::types::{has_chinese, Loader};

/// 校验翻译是否合格：非空、含简体中文、占位符集合与源一致
pub fn validate_translation(source: &str, translation: &str) -> bool {
    let t = translation.trim();
    if t.is_empty() || !has_chinese(t) {
        return false;
    }
    extract_protected_tokens(source) == extract_protected_tokens(t)
}

/// 从文本中提取第一个完整 JSON 对象（括号匹配，跳过字符串内的 `{`/`}`）
///
/// 容错：模型可能在 JSON 对象后追加解释文本，`rfind('}')` 会取到解释里的 `}`，
/// 导致截取范围混入非法内容。括号匹配从首个 `{` 开始，找到与之配对的 `}`。
pub fn extract_json_object(content: &str) -> Option<&str> {
    let start = content.find('{')?;
    let bytes = content.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else {
            match b {
                b'"' => in_string = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&content[start..=i]);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// 从 AI 响应中剥离围栏并提取翻译 JSON
///
/// 容错：去掉 ```json 围栏后取首个 `{` 到末个 `}` 之间的内容再解析。
pub fn parse_translations_response(content: &str) -> Result<Vec<(String, String)>, String> {
    let stripped = strip_json_fences(content);
    let json_str = extract_json_object(stripped).ok_or("AI 响应中未找到 JSON 对象")?;
    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("解析翻译 JSON 失败: {e}"))?;
    let translations = value
        .get("translations")
        .and_then(|v| v.as_array())
        .ok_or("AI 响应缺少 translations 数组")?;
    let mut result = Vec::new();
    for item in translations {
        let key = item
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let translation = item
            .get("translation")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if !key.is_empty() {
            result.push((key, translation));
        }
    }
    Ok(result)
}

/// 剥离 markdown 代码围栏（```json ... ```）
pub fn strip_json_fences(content: &str) -> &str {
    let trimmed = content.trim();
    trimmed
        .strip_prefix("```")
        .and_then(|rest| {
            let body = rest.strip_suffix("```")?;
            Some(
                body.trim()
                    .trim_start_matches("json\n")
                    .trim_start_matches("json"),
            )
        })
        .unwrap_or(trimmed)
}

/// 构造批量翻译用户消息
///
/// 模板外置在 `resources/prompts/mod_translation_user.md`（编译期内嵌），
/// 以 `{data}` 占位符注入待翻译条目 JSON 数据；模板缺失时使用内置兜底。
pub fn build_translation_user_prompt(
    loader: Loader,
    mod_ids: &[String],
    namespace: &str,
    entries: &[(String, String)],
) -> String {
    let entries_json: Vec<serde_json::Value> = entries
        .iter()
        .map(|(key, source)| serde_json::json!({ "key": key, "source": source }))
        .collect();
    let data = serde_json::json!({
        "loader": loader.as_str(),
        "modIds": mod_ids,
        "namespace": namespace,
        "entries": entries_json,
    })
    .to_string();
    let template = crate::resources::read_resource("prompts/mod_translation_user.md")
        .unwrap_or_else(|_| {
            "请翻译 entries 中的条目，严格按此格式返回 JSON：{{\"translations\":[{{\"key\":\"原key\",\"translation\":\"简体中文译文\"}}]}}，key 与 entries 一一对应，不得遗漏或新增。\n\n{data}"
                .to_string()
        });
    template.replace("{data}", &data)
}

#[cfg(test)]
#[path = "prompt_test.rs"]
mod tests;
