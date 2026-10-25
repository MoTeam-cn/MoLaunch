//! class 常量池解析 / UTF8 改写 / 候选聚合的单元测试

use super::*;

/// 常量池构造脚本
enum Spec<'a> {
    Utf8(&'a str),
    Class(u16),
    String(u16),
    Long(i64),
    Integer(i32),
}

/// 按脚本拼出一个最小合法 class 文件（常量池之后补空的最小类结构）
fn class_file(specs: &[Spec<'_>]) -> Vec<u8> {
    let mut pool: Vec<u8> = Vec::new();
    let mut slots: u16 = 0;
    for spec in specs {
        match spec {
            Spec::Utf8(text) => {
                pool.push(1);
                pool.extend_from_slice(&(text.len() as u16).to_be_bytes());
                pool.extend_from_slice(text.as_bytes());
                slots += 1;
            }
            Spec::Class(index) => {
                pool.push(7);
                pool.extend_from_slice(&index.to_be_bytes());
                slots += 1;
            }
            Spec::String(index) => {
                pool.push(8);
                pool.extend_from_slice(&index.to_be_bytes());
                slots += 1;
            }
            Spec::Long(value) => {
                pool.push(5);
                pool.extend_from_slice(&value.to_be_bytes());
                slots += 2;
            }
            Spec::Integer(value) => {
                pool.push(3);
                pool.extend_from_slice(&value.to_be_bytes());
                slots += 1;
            }
        }
    }

    let mut bytes: Vec<u8> = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x41];
    bytes.extend_from_slice(&(slots + 1).to_be_bytes());
    bytes.extend_from_slice(&pool);
    // access_flags / this_class / super_class / interfaces / fields / methods / attributes
    bytes.extend_from_slice(&[0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    bytes
}

/// 一把钻石剑 + 一个两槽位常量
fn sword_class() -> Vec<u8> {
    class_file(&[
        Spec::Utf8("Diamond Sword"),
        Spec::Class(1),
        Spec::Utf8("Diamond"),
        Spec::String(1),
        Spec::Long(0x0123_4567_89AB_CDEF),
        Spec::Utf8("Mythril Blade"),
    ])
}

#[test]
fn test_parse_utf8_entries_in_pool_order() {
    let bytes = sword_class();
    let entries = parse_class_constant_pool(&bytes).expect("pool should parse");

    let texts: Vec<&str> = entries.iter().map(|(text, _, _)| text.as_str()).collect();
    assert_eq!(texts, vec!["Diamond Sword", "Diamond", "Mythril Blade"]);

    // 每个区间必须正好盖住 tag + 长度 + 内容
    for (text, start, end) in &entries {
        assert_eq!(bytes[*start], 1, "span should start at the UTF8 tag");
        let declared = u16::from_be_bytes([bytes[*start + 1], bytes[*start + 2]]) as usize;
        assert_eq!(*end - *start, 3 + declared);
        assert_eq!(&bytes[*start + 3..*end], text.as_bytes());
    }
}

#[test]
fn test_string_constants_only_include_string_targets() {
    let bytes = sword_class();
    assert_eq!(class_string_constants(&bytes), vec!["Diamond Sword"]);
}

#[test]
fn test_two_slot_constant_keeps_slot_accounting() {
    // Long 占两个槽位；若槽位记账有误，后面的 Utf8 会读飞或提前结束
    let bytes = class_file(&[
        Spec::Utf8("First"),
        Spec::Long(-1),
        Spec::Utf8("Second"),
        Spec::Integer(7),
        Spec::String(4),
    ]);
    let entries = parse_class_constant_pool(&bytes).expect("pool should parse");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].0, "Second");
    assert_eq!(class_string_constants(&bytes), vec!["Second"]);
}

#[test]
fn test_rejects_non_class_payload() {
    let error = parse_class_constant_pool(b"PK\u{3}\u{4}not-a-class").unwrap_err();
    assert!(error.contains("magic"), "unexpected error: {error}");
    assert!(parse_class_constant_pool(&[]).is_err());
}

#[test]
fn test_rejects_truncated_pool() {
    let mut bytes = sword_class();
    bytes.truncate(20);
    let error = parse_class_constant_pool(&bytes).unwrap_err();
    assert!(error.contains("truncated"), "unexpected error: {error}");
}

#[test]
fn test_rejects_unknown_constant_tag() {
    let mut bytes: Vec<u8> = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x41, 0x00, 0x02];
    bytes.push(99);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    let error = parse_class_constant_pool(&bytes).unwrap_err();
    assert!(error.contains("99"), "unexpected error: {error}");
}

#[test]
fn test_replace_utf8_rewrites_target_only() {
    let bytes = sword_class();
    let rewritten = replace_class_utf8(&bytes, "Diamond Sword", "钻石剑").expect("rewrite should work");

    assert!(rewritten.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE]));
    assert_eq!(class_string_constants(&rewritten), vec!["钻石剑"]);

    let entries = parse_class_constant_pool(&rewritten).expect("rewritten pool should parse");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[1].0, "Diamond");
    assert_eq!(entries[2].0, "Mythril Blade");
    // 只重建被替换条目，其余字节逐字节一致
    assert_eq!(&rewritten[rewritten.len() - 14..], &bytes[bytes.len() - 14..]);
}

#[test]
fn test_replace_utf8_is_noop_for_unknown_text() {
    let bytes = sword_class();
    assert_eq!(replace_class_utf8(&bytes, "Iron Ingot", "铁锭").unwrap(), bytes);
    assert_eq!(replace_class_utf8(&bytes, "Diamond", "Diamond").unwrap(), bytes);
}

#[test]
fn test_replace_utf8_rejects_oversized_replacement() {
    let bytes = sword_class();
    let huge = "x".repeat(70_000);
    assert!(replace_class_utf8(&bytes, "Diamond Sword", &huge).is_err());
}

#[test]
fn test_classify_candidate_text_shapes() {
    assert_eq!(classify_class_text("Diamond Sword"), (true, "natural_language"));
    assert_eq!(classify_class_text("MythrilAxe"), (true, "display_word"));
    assert_eq!(classify_class_text("minecraft:diamond_sword"), (false, "structural"));
    assert_eq!(classify_class_text("DIAMOND_TIER"), (false, "constant"));
    assert_eq!(classify_class_text("已本地化"), (false, "already_localized"));
    assert_eq!(classify_class_text("%d / %d"), (false, "format_only"));
}

#[test]
fn test_discover_aggregates_candidates_across_files() {
    let workspace = std::env::temp_dir().join(format!("mt-class-agg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(workspace.join("assets/pkg")).unwrap();
    let bytes = sword_class();
    std::fs::write(workspace.join("assets/pkg/a.class"), &bytes).unwrap();
    std::fs::write(workspace.join("assets/pkg/b.class"), &bytes).unwrap();

    let candidates = discover_class_candidates(&workspace);
    let _ = std::fs::remove_dir_all(&workspace);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].text, "Diamond Sword");
    assert_eq!(candidates[0].occurrences, 2);
    assert_eq!(candidates[0].paths.len(), 2);
    assert_eq!(candidates[0].id.len(), 24);
}

#[test]
fn test_deterministic_exclusions_by_host_and_text() {
    assert_eq!(
        deterministic_class_exclusion_reason(
            "com/example/map/file/RegionReader.class",
            "IOException while reading the region!"
        ),
        Some("internal_diagnostic")
    );
    assert_eq!(
        deterministic_class_exclusion_reason(
            "com/example/map/gui/RegionScreen.class",
            "Region file is unreadable. Retry?"
        ),
        None
    );
    assert_eq!(
        deterministic_class_exclusion_reason("com/example/Foo.class", "com.example.Foo"),
        Some("java_class_name")
    );
    assert_eq!(
        deterministic_class_exclusion_reason("com/example/Foo.class", "^a+\\d{2,3}$"),
        Some("regular_expression")
    );
}
