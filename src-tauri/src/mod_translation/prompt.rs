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

/// 从 AI 响应中剥离围栏并提取翻译 JSON
///
/// 容错：去掉 ```json 围栏后取首个 `{` 到末个 `}` 之间的内容再解析。
pub fn parse_translations_response(content: &str) -> Result<Vec<(String, String)>, String> {
    let stripped = strip_json_fences(content);
    let start = stripped.find('{').ok_or("AI 响应中未找到 JSON 对象")?;
    let end = stripped.rfind('}').ok_or("AI 响应中未找到 JSON 对象")?;
    let json_str = &stripped[start..=end];
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
    serde_json::json!({
        "loader": loader.as_str(),
        "modIds": mod_ids,
        "namespace": namespace,
        "entries": entries_json,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_fences_handles_code_block() {
        let input = "```json\n{\"translations\":[{\"key\":\"a\",\"translation\":\"甲\"}]}\n```";
        assert!(parse_translations_response(input).is_ok());
    }

    #[test]
    fn parse_response_without_fences() {
        let input = "{\"translations\":[{\"key\":\"a\",\"translation\":\"甲\"}]}";
        let parsed = parse_translations_response(input).unwrap();
        assert_eq!(parsed, vec![("a".to_string(), "甲".to_string())]);
    }

    #[test]
    fn placeholders_preserved() {
        assert!(validate_translation("Diamond %s", "钻石 %s"));
        assert!(validate_translation("Hello {0}", "你好 {0}"));
        assert!(validate_translation("§aGreen", "§a绿色"));
        assert!(!validate_translation("Diamond %s", "钻石"));
        assert!(!validate_translation("Hello", "Hello"));
        assert!(!validate_translation("Diamond", "钻石 %s"));
    }

    #[test]
    fn protected_tokens_extraction() {
        let tokens = extract_protected_tokens("a %1$s b {2} c {{x}} d §6 e \\n");
        assert!(tokens.iter().any(|t| t == "%1$s"));
        assert!(tokens.iter().any(|t| t == "{2}"));
        assert!(tokens.iter().any(|t| t == "{{x}}"));
        assert!(tokens.iter().any(|t| t == "§6"));
        assert!(tokens.iter().any(|t| t == "\\n"));
    }
}
