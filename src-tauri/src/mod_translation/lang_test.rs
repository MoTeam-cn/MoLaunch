//! 语言文件读写测试（规格 §2）

use std::collections::BTreeMap;

use super::free_text::{align_free_text, render_localized_text, snapshot_free_text};
use super::json::{read_json_lang, write_json_lang};
use super::keyvalue::{parse_keyvalue, write_keyvalue};
use super::structured::{apply_structured_strings, collect_structured_strings};

#[test]
fn test_json_lang_flattens_nested_strings() {
    let content = r#"{"a":"one","nested":{"b":"two","num":3},"arr":["x"]}"#;
    let entries = read_json_lang(content).expect("read");
    assert_eq!(entries.get("a").map(String::as_str), Some("one"));
    assert_eq!(entries.get("nested.b").map(String::as_str), Some("two"));
    assert!(!entries.contains_key("nested.num"));
    assert!(!entries.contains_key("arr"));
}

#[test]
fn test_json_lang_write_keeps_untouched_fields() {
    let content = r#"{"a":"one","nested":{"b":"two","num":3}}"#;
    let mut translations = BTreeMap::new();
    translations.insert("nested.b".to_string(), "二".to_string());
    let output = write_json_lang(content, &translations).expect("write");
    let reparsed = read_json_lang(&output).expect("reparse");
    assert_eq!(reparsed.get("nested.b").map(String::as_str), Some("二"));
    assert_eq!(reparsed.get("a").map(String::as_str), Some("one"));
    assert!(output.contains("\"num\": 3"));
}

#[test]
fn test_structured_strings_use_json_pointer_and_escape_keys() {
    let content = r#"{"a/b":{"c~d":"text"}}"#;
    let entries = collect_structured_strings(content).expect("collect");
    assert_eq!(entries.get("/a~1b/c~0d").map(String::as_str), Some("text"));

    let mut translations = BTreeMap::new();
    translations.insert("/a~1b/c~0d".to_string(), "中文".to_string());
    let output = apply_structured_strings(content, &translations).expect("apply");
    assert!(output.contains("\"中文\""));
}

#[test]
fn test_keyvalue_parsing_skips_comments_and_handles_escaped_separator() {
    let content = "# comment\n\nkey=value\n  other :  spaced \npath\\=key=1\n";
    let (entries, lines) = parse_keyvalue(content);
    assert_eq!(lines.len(), 5);
    let keys: Vec<&str> = entries.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(keys, vec!["key", "other", "path\\=key"]);
    assert_eq!(entries[1].1, "spaced");
    assert_eq!(entries[2].1, "1");
}

#[test]
fn test_keyvalue_write_preserves_layout() {
    let content = "# keep me\n  greeting=hello\nplain=value";
    let (_, lines) = parse_keyvalue(content);
    let mut translations = BTreeMap::new();
    translations.insert("greeting".to_string(), "你好".to_string());
    let output = write_keyvalue(&lines, &translations);
    assert!(output.contains("# keep me"));
    assert!(output.contains("  greeting=你好"));
    assert!(output.contains("plain=value"));
}

#[test]
fn test_free_text_snapshot_detects_bom_and_crlf() {
    let content = "\u{feff}line1\r\nline2\r\n";
    let snapshot = snapshot_free_text(content);
    assert!(snapshot.has_bom);
    assert_eq!(snapshot.eol, "\r\n");
    assert!(snapshot.trailing_newline);
    assert_eq!(snapshot.source_lines, vec!["line1", "line2"]);
    assert_eq!(render_localized_text(&snapshot), content);
}

#[test]
fn test_free_text_align_only_replaces_translated_lines() {
    let lines = vec!["first".to_string(), "".to_string(), "third".to_string()];
    let mut translations = BTreeMap::new();
    translations.insert("/lines/000000".to_string(), "第一".to_string());
    translations.insert("/lines/000001".to_string(), "不该写入".to_string());
    translations.insert("/lines/000002".to_string(), "  ".to_string());
    let aligned = align_free_text(&lines, &translations);
    assert_eq!(aligned[0], "第一");
    assert_eq!(aligned[1], "");
    assert_eq!(aligned[2], "third");
}
