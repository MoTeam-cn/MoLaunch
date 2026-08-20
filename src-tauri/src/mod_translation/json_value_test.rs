//! 保序 JSON 单元测试（经 json_value.rs 的 #[path] 子模块引入）

use super::*;

#[test]
fn parse_preserves_key_order() {
    let root = JsonValue::parse(r#"{"z":"1","a":"2","m":"3"}"#).unwrap();
    let rendered = root.render_pretty();
    let z = rendered.find("\"z\"").unwrap();
    let a = rendered.find("\"a\"").unwrap();
    let m = rendered.find("\"m\"").unwrap();
    assert!(z < a && a < m, "键序必须保留: {rendered}");
}

#[test]
fn set_pointer_writes_nested_value() {
    let mut root = JsonValue::parse(r#"{"a":{"b":[{"c":"old"}]}}"#).unwrap();
    root.set_pointer("/a/b/0/c", "new".to_string()).unwrap();
    assert!(root.render_pretty().contains("\"c\": \"new\""));
}

#[test]
fn render_integers_without_decimal() {
    let root = JsonValue::parse(r#"{"n":10,"f":1.5}"#).unwrap();
    let rendered = root.render_pretty();
    assert!(rendered.contains("\"n\": 10") && rendered.contains("\"f\": 1.5"));
}
