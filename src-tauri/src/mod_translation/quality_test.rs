//! 占位符与审计测试（规格 §5.4）

use std::collections::BTreeMap;

use super::{
    audit_invariants, audit_semantic, extract_protected_tokens, is_passthrough_entry,
    language_work_weight, normalize_model_translation, requires_work, validate_protected_tokens,
    AuditSeverity,
};

#[test]
fn test_printf_placeholders_must_survive() {
    assert!(validate_protected_tokens("Spawn %d zombies", "生成 %d 只僵尸").is_none());
    assert!(validate_protected_tokens("Spawn %d zombies", "生成 %s 只僵尸").is_some());
    assert!(validate_protected_tokens("%1$s took %2$d damage", "%1$s 受到 %2$d 点伤害").is_none());
    assert!(validate_protected_tokens("50% done", "完成 50%").is_some());
}

#[test]
fn test_format_codes_and_escapes_must_survive() {
    assert!(validate_protected_tokens("§aGreen text", "§a绿色文字").is_none());
    assert!(validate_protected_tokens("line\\nbreak", "换行\\n测试").is_none());
    assert!(validate_protected_tokens("line\\nbreak", "换行测试").is_some());
    assert!(validate_protected_tokens("{0} of {1}", "{0} / {1}").is_none());
}

#[test]
fn test_protected_tokens_are_sorted_with_duplicates_preserved() {
    let tokens = extract_protected_tokens("§a%s and %s and ${name}");
    assert_eq!(tokens, vec!["${name}", "%s", "%s", "§a"]);
}

#[test]
fn test_model_escapes_normalized_to_source_direction() {
    assert_eq!(
        normalize_model_translation("Rendering disabled: %s", "渲染已禁用：%s\\n请重新启用"),
        "渲染已禁用：%s\n请重新启用"
    );
    assert_eq!(
        normalize_model_translation("line\\nbreak", "第一行\n第二行"),
        "第一行\\n第二行"
    );
}

#[test]
fn test_invariants_catch_missing_and_placeholder_breaks() {
    let source = BTreeMap::from([("a".to_string(), "Spawn %d".to_string())]);
    let target = BTreeMap::from([("a".to_string(), "生成".to_string())]);
    let issues = audit_invariants(&source, &target);
    assert!(issues
        .iter()
        .any(|issue| issue.severity == AuditSeverity::Error && issue.key == "a"));

    let empty_target = BTreeMap::new();
    let missing = audit_invariants(&source, &empty_target);
    assert!(missing
        .iter()
        .any(|issue| issue.message.contains("缺少中文译文")));

    let extra = BTreeMap::from([("b".to_string(), "多余".to_string())]);
    let warnings = audit_invariants(&source, &extra);
    assert!(warnings
        .iter()
        .any(|issue| issue.severity == AuditSeverity::Warning));
}

#[test]
fn test_official_terms_and_consistent_translation() {
    let source = BTreeMap::from([
        ("k1".to_string(), "Iron Bars".to_string()),
        ("k2".to_string(), "Wrench".to_string()),
        ("k3".to_string(), "Wrench".to_string()),
    ]);
    let target = BTreeMap::from([
        ("k1".to_string(), "铁条".to_string()),
        ("k2".to_string(), "扳手".to_string()),
        ("k3".to_string(), "螺丝刀".to_string()),
    ]);
    let issues = audit_semantic(&source, &target);
    assert!(issues.iter().any(|issue| issue.message.contains("铁栏杆")));
    assert!(issues.iter().any(|issue| issue.message.contains("种译法")));
}

#[test]
fn test_passthrough_entries_are_detected() {
    assert!(is_passthrough_entry("a", "https://example.com/a"));
    assert!(is_passthrough_entry("a", "/give @p minecraft:stone"));
    assert!(is_passthrough_entry("a", "mymod:block/deep"));
    assert!(is_passthrough_entry("item.music_disc.cat", "C418 - Cat"));
    assert!(!is_passthrough_entry("a", "Press any key"));
    assert!(!is_passthrough_entry("item.name", "C418 - Cat"));
}

#[test]
fn test_requires_work_covers_empty_identical_and_localized() {
    assert!(!requires_work("a", "", None));
    assert!(requires_work("a", "Press any key", None));
    assert!(!requires_work("a", "Press any key", Some("按任意键")));
    assert!(requires_work("a", "Press any key", Some("Press any key")));
    assert!(!requires_work("a", "https://example.com", None));
}

#[test]
fn test_language_work_weight_grows_with_length_and_placeholders() {
    let short = language_work_weight("Hi");
    let long = language_work_weight(&"word ".repeat(40));
    let with_tokens = language_work_weight("Spawn %d zombies at %s %s");
    assert!(long > short);
    assert!(with_tokens > language_work_weight("Spawn zombies at point"));
}
