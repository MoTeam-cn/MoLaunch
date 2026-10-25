//! 模组中文名决策链测试

use super::resolve_mod_name;

#[test]
fn test_resolve_mod_name_embedded_chinese_wins() {
    let result = resolve_mod_name(
        &["Macaw 的窗户".to_string()],
        &["mcwwindows".to_string()],
        "mcwwindows-1.0.jar",
        None,
    );
    assert_eq!(result.source, "embedded_chinese");
    assert_eq!(result.name, "Macaw 的窗户");
}

#[test]
fn test_resolve_mod_name_uses_known_table() {
    let result = resolve_mod_name(&[], &["mekanism".to_string()], "mekanism-1.20.1.jar", None);
    assert_eq!(result.source, "known_chinese");
    assert_eq!(result.name, "通用机械");
}

#[test]
fn test_resolve_mod_name_translates_possessive_label() {
    let result = resolve_mod_name(&[], &[], "Alex's Caves-1.2.jar", None);
    assert_eq!(result.source, "translated_filename");
    assert_eq!(result.name, "Alex 的洞穴");
}

#[test]
fn test_resolve_mod_name_falls_back_to_mod_id() {
    let result = resolve_mod_name(&[], &["weird_mod_id".to_string()], "download.jar", None);
    assert_eq!(result.source, "mod_id");
    assert_eq!(result.name, "weird_mod_id");
}

#[test]
fn test_resolve_mod_name_prefers_ai_recommendation_over_known_table() {
    let result = resolve_mod_name(
        &["Generic Mod".to_string()],
        &["mekanism".to_string()],
        "mekanism-1.20.1.jar",
        Some("机械动力".to_string()),
    );
    assert_eq!(result.source, "ai_recommended");
    assert_eq!(result.name, "机械动力");
}

#[test]
fn test_resolve_mod_name_rejects_generic_names() {
    let result = resolve_mod_name(
        &["模组".to_string()],
        &[],
        "unknown.jar",
        Some("未知".to_string()),
    );
    assert_ne!(result.source, "ai_recommended");
    assert_ne!(result.source, "embedded_chinese");
}
