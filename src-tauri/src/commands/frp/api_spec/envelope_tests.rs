//! envelope 单元测试

use super::*;
use serde_json::json;

fn env(success_field: &str, success_value: Value, error_field: &str, data_field: &str) -> Envelope {
    Envelope {
        success_field: Some(success_field.to_string()),
        success_value: Some(success_value),
        error_field: Some(error_field.to_string()),
        data_field: Some(data_field.to_string()),
    }
}

#[test]
fn test_success_bool() {
    let e = env("$.flag", json!(true), "$.msg", "$.data");
    let resp = json!({ "flag": true, "data": {} });
    assert!(is_success(&resp, Some(&e)));
}

#[test]
fn test_success_number_string() {
    let e = env("$.code", json!(200), "$.msg", "$.data");
    let resp = json!({ "code": "200", "data": {} });
    assert!(is_success(&resp, Some(&e)));
}

#[test]
fn test_failure() {
    let e = env("$.flag", json!(true), "$.msg", "$.data");
    let resp = json!({ "flag": false, "msg": "未授权" });
    assert!(!is_success(&resp, Some(&e)));
    assert_eq!(extract_error(&resp, Some(&e)), Some("未授权".to_string()));
}

#[test]
fn test_extract_data() {
    let e = env("$.flag", json!(true), "$.msg", "$.data");
    let resp = json!({ "flag": true, "data": { "id": 1 } });
    let data = extract_data(&resp, Some(&e), None).unwrap();
    assert_eq!(data, Some(json!({ "id": 1 })));
}
