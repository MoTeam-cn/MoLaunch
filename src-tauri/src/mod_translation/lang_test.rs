//! 语言文件读写单元测试（经 lang.rs 的 #[path] 子模块引入）

use std::collections::BTreeMap;

use super::*;

#[test]
fn structured_json_keeps_key_order_and_scalars() {
    let original = "{\"z\": \"Hello\", \"a\": {\"n\": 10, \"b\": true, \"title\": \"Hi\"}}";
    let translations = BTreeMap::from([
        ("/z".to_string(), "你好".to_string()),
        ("/a/title".to_string(), "标题".to_string()),
    ]);
    let out = apply_structured_strings(original, &translations).unwrap();
    assert!(
        out.find("\"z\"").unwrap() < out.find("\"a\"").unwrap(),
        "{out}"
    );
    assert!(
        out.contains("\"n\": 10") && out.contains("\"b\": true"),
        "{out}"
    );
    assert!(out.contains("\"z\": \"你好\"") && out.contains("\"title\": \"标题\""));
}

#[test]
fn free_text_snapshot_round_trips_bom_eol() {
    let content = "\u{feff}line1\r\nline2\r\n";
    let snap = snapshot_free_text(content);
    assert!(snap.has_bom);
    assert_eq!(snap.eol, "\r\n");
    assert!(snap.trailing_newline);
    assert_eq!(snap.source_lines, vec!["line1", "line2"]);
    assert_eq!(render_localized_text(&snap), content);
}

#[test]
fn render_localized_text_falls_back_to_source() {
    let mut snap = snapshot_free_text("a\nb\nc\n");
    snap.target_lines = vec!["甲".to_string(), String::new()];
    assert_eq!(render_localized_text(&snap), "甲\nb\nc\n");
}

#[test]
fn read_localized_target_maps_lines_to_keys() {
    let dir = std::env::temp_dir();
    let target = dir.join("mo_launch_test_zh_cn.txt");
    std::fs::write(&target, "\u{feff}甲\n\n乙\n").unwrap();
    let map = read_localized_target(&dir, &target);
    std::fs::remove_file(&target).ok();
    assert_eq!(map.get("/lines/000000").map(String::as_str), Some("甲"));
    assert_eq!(map.get("/lines/000002").map(String::as_str), Some("乙"));
    assert!(!map.contains_key("/lines/000001"));
}

#[test]
fn parse_keyvalue_handles_escaped_separators() {
    let content = "a=1\nb:two\nc\\=x=3\nd\\:y=4\n";
    let (kv, _) = parse_keyvalue(content);
    let map: BTreeMap<String, String> = kv.into_iter().collect();
    assert_eq!(map.get("a").map(String::as_str), Some("1"));
    assert_eq!(map.get("b").map(String::as_str), Some("two"));
    assert_eq!(map.get("c\\=x").map(String::as_str), Some("3"));
    assert_eq!(map.get("d\\:y").map(String::as_str), Some("4"));
}
