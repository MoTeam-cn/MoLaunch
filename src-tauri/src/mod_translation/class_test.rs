//! class 常量池解析/候选/改写单元测试（经 class.rs 的 #[path] 子模块引入）

use super::*;

/// 最小合法 class 文件：3 个 Utf8 + 2 个 Class + 1 个 String 引用
fn fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 0x3d]);
    bytes.extend_from_slice(&[0, 7]); // constant_pool_count = 7（索引 1-6）
    for value in ["Hello", "World", "Iron Ingot"] {
        bytes.push(1);
        bytes.extend_from_slice(&(value.len() as u16).to_be_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    bytes.extend_from_slice(&[7, 0, 1]); // Class -> 1
    bytes.extend_from_slice(&[7, 0, 2]); // Class -> 2
    bytes.extend_from_slice(&[8, 0, 3]); // String -> 3
    bytes.extend_from_slice(&[0, 1]); // access_flags
    bytes.extend_from_slice(&[0, 4]); // this_class
    bytes.extend_from_slice(&[0, 5]); // super_class
    bytes.extend_from_slice(&[0, 0]); // interfaces_count
    bytes.extend_from_slice(&[0, 0]); // fields_count
    bytes.extend_from_slice(&[0, 0]); // methods_count
    bytes.extend_from_slice(&[0, 0]); // attributes_count
    bytes
}

#[test]
fn parses_constant_pool_and_string_refs() {
    let bytes = fixture();
    let entries = parse_class_constant_pool(&bytes).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].0, "Hello");
    assert_eq!(entries[2].0, "Iron Ingot");
    assert_eq!(class_string_constants(&bytes), vec!["Iron Ingot"]);
}

#[test]
fn classifies_candidate_text() {
    assert_eq!(
        classify_class_text("Iron Ingot"),
        (true, "natural_language")
    );
    assert_eq!(
        classify_class_text("Welcome to the server"),
        (true, "natural_language")
    );
    assert_eq!(classify_class_text("IronIngot"), (true, "display_word"));
    assert_eq!(
        classify_class_text("minecraft:iron_ingot"),
        (false, "structural")
    );
    assert_eq!(classify_class_text("ITEM_REGISTRY"), (false, "constant"));
    assert_eq!(classify_class_text("已翻译"), (false, "already_localized"));
    assert_eq!(classify_class_text("%s: %s"), (false, "format_only"));
}

#[test]
fn replaces_utf8_and_preserves_structure() {
    let bytes = fixture();
    let rewritten = replace_class_utf8(&bytes, "Iron Ingot", "铁锭").unwrap();
    assert!(rewritten.starts_with(&[0xca, 0xfe, 0xba, 0xbe]));
    let entries = parse_class_constant_pool(&rewritten).unwrap();
    assert_eq!(entries.len(), 3);
    assert!(entries.iter().any(|(t, _, _)| t == "铁锭"));
    assert_eq!(class_string_constants(&rewritten), vec!["铁锭"]);
    assert_eq!(replace_class_utf8(&bytes, "Nope", "x").unwrap(), bytes);
}

#[test]
fn aggregates_candidates_across_files() {
    let dir = std::env::temp_dir().join(format!("mt-class-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bytes = fixture();
    std::fs::write(dir.join("a.class"), &bytes).unwrap();
    std::fs::write(dir.join("b.class"), &bytes).unwrap();
    let candidates = discover_class_candidates(&dir);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].text, "Iron Ingot");
    assert_eq!(candidates[0].occurrences, 2);
    assert_eq!(candidates[0].paths.len(), 2);
    assert_eq!(candidates[0].id.len(), 24);
}

#[test]
fn deterministic_exclusions() {
    assert_eq!(
        deterministic_class_exclusion_reason(
            "xaero/map/file/MapProcessor.class",
            "IOException trying to detect map files!"
        ),
        Some("internal_diagnostic")
    );
    assert_eq!(
        deterministic_class_exclusion_reason(
            "xaero/map/gui/GuiMap.class",
            "Failed to load your map. Retry?"
        ),
        None
    );
    assert_eq!(
        deterministic_class_exclusion_reason("a/b/SomeClass.class", "com.example.SomeClass"),
        Some("java_class_name")
    );
}
