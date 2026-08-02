//! jsonpath 单元测试

use super::*;
use serde_json::json;

#[test]
fn test_extract_simple() {
    let v = json!({ "access_token": "abc123" });
    assert_eq!(extract(&v, "$.access_token"), Some(json!("abc123")));
}

#[test]
fn test_extract_nested() {
    let v = json!({ "data": { "config": "frpc content" } });
    assert_eq!(extract(&v, "$.data.config"), Some(json!("frpc content")));
}

#[test]
fn test_extract_array_flat() {
    let v = json!({ "data": [{ "id": 1 }, { "id": 2 }] });
    let arr = extract_array(&v, "$.data[*]").unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["id"], 1);
}

#[test]
fn test_extract_array_nested() {
    let v = json!({
        "data": [
            { "proxies": [{ "id": "a" }, { "id": "b" }] },
            { "proxies": [{ "id": "c" }] }
        ]
    });
    let arr = extract_array(&v, "$.data[*].proxies[*]").unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["id"], "a");
    assert_eq!(arr[2]["id"], "c");
}

#[test]
fn test_extract_missing() {
    let v = json!({ "a": 1 });
    assert_eq!(extract(&v, "$.b"), None);
}
