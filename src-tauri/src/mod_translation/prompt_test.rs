//! 提示词构造与响应解析测试（经 prompt.rs 的 #[path] 子模块引入）

use super::*;

const FENCED: &str = "```json\n{\"translations\":[{\"key\":\"a\",\"translation\":\"甲\"}]}\n```";

#[test]
fn test_parse_response_strips_code_fences() {
    assert!(parse_translations_response(FENCED).is_ok());
}

#[test]
fn test_parse_response_without_fences() {
    let parsed =
        parse_translations_response("{\"translations\":[{\"key\":\"a\",\"translation\":\"甲\"}]}")
            .expect("parse");
    assert_eq!(parsed, vec![("a".to_string(), "甲".to_string())]);
}

#[test]
fn test_parse_response_ignores_trailing_explanation() {
    let raw = "{\"translations\":[{\"key\":\"a\",\"translation\":\"甲\"}]} 以上是结果，若有疑问请告知 {备注}";
    let parsed = parse_translations_response(raw).expect("parse");
    assert_eq!(parsed.len(), 1);
}

#[test]
fn test_validate_translation_requires_placeholders_and_chinese() {
    assert!(validate_translation("Diamond %s", "钻石 %s"));
    assert!(validate_translation("Hello {0}", "你好 {0}"));
    assert!(validate_translation("§aGreen", "§a绿色"));
    assert!(!validate_translation("Diamond %s", "钻石"));
    assert!(!validate_translation("Hello", "Hello"));
    assert!(!validate_translation("Diamond", "钻石 %s"));
}

#[test]
fn test_protected_tokens_cover_all_placeholder_shapes() {
    let tokens = extract_protected_tokens("a %1$s b {2} c {{x}} d §6 e \\n");
    for expected in ["%1$s", "{2}", "{{x}}", "§6", "\\n"] {
        assert!(
            tokens.iter().any(|token| token == expected),
            "缺少 {expected}"
        );
    }
}

#[test]
fn test_user_prompt_injects_entries() {
    let prompt = build_translation_user_prompt(
        Loader::Fabric,
        &["demo".to_string()],
        "demo",
        &[("a".to_string(), "Diamond".to_string())],
    );
    assert!(prompt.contains("translations"));
    assert!(prompt.contains("\"key\":\"a\""));
    assert!(prompt.contains("\"source\":\"Diamond\""));
}
