//! 保序 JSON 解析 / 渲染 / 指针写入测试（规格 §2）

use super::JsonValue;

#[test]
fn test_parse_keeps_object_key_order() {
    let value = JsonValue::parse(r#"{"b":1,"a":2}"#).expect("parse");
    let JsonValue::Object(entries) = value else {
        panic!("expected an object");
    };
    let keys: Vec<&str> = entries.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(keys, vec!["b", "a"]);
}

#[test]
fn test_render_pretty_round_trips() {
    let source = r#"{"pack":{"description":"demo"},"n":3,"list":[1,2]}"#;
    let rendered = JsonValue::parse(source).expect("parse").render_pretty();
    let reparsed = JsonValue::parse(&rendered).expect("reparse");
    assert_eq!(reparsed.render_pretty(), rendered);
    assert!(rendered.contains("  \"pack\": {"));
}

#[test]
fn test_render_writes_integers_without_fraction() {
    let rendered = JsonValue::parse(r#"{"a":1.0,"b":2.5}"#)
        .expect("parse")
        .render_pretty();
    assert!(rendered.contains("\"a\": 1"));
    assert!(rendered.contains("\"b\": 2.5"));
}

#[test]
fn test_parse_handles_escapes_and_surrogate_pairs() {
    let value = JsonValue::parse(r#""a\\\"b\\n\\u0041\\ud83e\\udd8e""#).expect("parse");
    let JsonValue::String(text) = value else {
        panic!("expected a string");
    };
    assert_eq!(text, "a\"b\nA🦎");
}

#[test]
fn test_parse_rejects_trailing_content() {
    assert!(JsonValue::parse(r#"{"a":1} extra"#).is_err());
}

#[test]
fn test_set_pointer_updates_nested_object() {
    let mut value = JsonValue::parse(r#"{"a":{"b":"old"},"list":["x"]}"#).expect("parse");
    value
        .set_pointer("/a/b", "新值".to_string())
        .expect("object pointer");
    value
        .set_pointer("/list/0", "替换".to_string())
        .expect("array pointer");
    let rendered = value.render_pretty();
    assert!(rendered.contains("\"b\": \"新值\""));
    assert!(rendered.contains("\"替换\""));
}

#[test]
fn test_set_pointer_rejects_missing_and_scalar_paths() {
    let mut value = JsonValue::parse(r#"{"a":"text"}"#).expect("parse");
    assert!(value.set_pointer("/missing", "x".to_string()).is_err());
    assert!(value.set_pointer("/a/b", "x".to_string()).is_err());
    assert!(value.set_pointer("no-slash", "x".to_string()).is_err());
}
