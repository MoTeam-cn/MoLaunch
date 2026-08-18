//! 提示词构造与响应解析单元测试（经 prompt.rs 的 #[path] 子模块引入）

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

#[test]
fn user_prompt_injects_data() {
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
