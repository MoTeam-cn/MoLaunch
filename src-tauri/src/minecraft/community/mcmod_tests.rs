//! mcmod 数据库单元测试

use super::*;

#[test]
fn test_parse_slug_part() {
    // @slug → 仅 Modrinth
    let (cf, mr) = parse_slug_part("@redpower2-core");
    assert_eq!(cf, None);
    assert_eq!(mr, Some("redpower2-core".to_string()));

    // slug@ → CF=MR=slug
    let (cf, mr) = parse_slug_part("industrial-craft@");
    assert_eq!(cf, Some("industrial-craft".to_string()));
    assert_eq!(mr, Some("industrial-craft".to_string()));

    // cf@mr → 双平台不同 slug
    let (cf, mr) = parse_slug_part("railcraft@railcraft-reborn");
    assert_eq!(cf, Some("railcraft".to_string()));
    assert_eq!(mr, Some("railcraft-reborn".to_string()));

    // slug → 仅 CurseForge
    let (cf, mr) = parse_slug_part("buildcraft");
    assert_eq!(cf, Some("buildcraft".to_string()));
    assert_eq!(mr, None);
}

#[test]
fn test_process_wildcard() {
    // * 替换为 (Slug 去横线首字母大写)
    let result = process_wildcard("林业*", "forestry@");
    assert!(
        result.contains("Forestry"),
        "wildcard should be replaced, got {}",
        result
    );
}

#[test]
fn test_extract_words() {
    let words = extract_words(
        "工业时代2 (Industrial Craft 2)",
        Some("industrial-craft"),
        Some("industrial-craft"),
    );
    assert!(
        words.contains(&"industrial".to_string()),
        "should contain industrial, got {:?}",
        words
    );
    assert!(
        words.contains(&"craft".to_string()),
        "should contain craft, got {:?}",
        words
    );
}

#[test]
fn test_extract_words_filters_stopwords() {
    // 停用词（the/of/mod/forge 等）应被过滤
    let words = extract_words("测试 (The Mod of Forge)", Some("the-mod-of-forge"), None);
    assert!(
        !words
            .iter()
            .any(|w| w == "the" || w == "mod" || w == "forge"),
        "stopwords should be filtered, got {:?}",
        words
    );
}

#[test]
fn test_parse_slug_part_empty() {
    let (cf, mr) = parse_slug_part("");
    assert_eq!(cf, None);
    assert_eq!(mr, None);

    // 只有 @
    let (cf, mr) = parse_slug_part("@");
    assert_eq!(cf, None);
    assert_eq!(mr, None);
}
