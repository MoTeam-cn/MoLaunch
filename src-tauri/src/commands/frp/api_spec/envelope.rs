//! 响应包裹解析（envelope）
//!
//! 不同厂商成功判断方式不同（如 `{flag:true, msg}` 或 `{success:true, message}`），
//! 通过 envelope 配置统一解析：
//! - successField + successValue：成功判断
//! - errorField：失败时错误消息
//! - dataField：数据位置
//!
//! 各接口可用 endpoints.*.envelope 覆盖全局 envelope。

use super::jsonpath;
use super::super::Envelope;
use serde_json::Value;

/// 判断响应是否成功
///
/// 优先使用接口级 envelope，其次全局 envelope。
/// successField 缺省时默认 HTTP 2xx 即成功（由调用方保证）。
pub fn is_success(response: &Value, envelope: Option<&Envelope>) -> bool {
    let Some(env) = resolve_envelope(envelope) else {
        return true; // 无 envelope 配置，默认成功（HTTP 状态码已校验）
    };

    if let Some(ref field) = env.success_field {
        let actual = jsonpath::extract(response, field);
        match (actual, &env.success_value) {
            (Some(val), Some(expected)) => values_equal(&val, expected),
            (Some(val), None) => val.as_bool().unwrap_or(false),
            (None, _) => false,
        }
    } else {
        true
    }
}

/// 提取错误消息
pub fn extract_error(response: &Value, envelope: Option<&Envelope>) -> Option<String> {
    let env = resolve_envelope(envelope)?;
    let field = env.error_field.clone()?;
    let val = jsonpath::extract(response, &field)?;
    match val {
        Value::String(s) if !s.is_empty() => Some(s),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// 提取数据字段
///
/// 优先使用接口级 envelope.dataField，其次全局 envelope.dataField，
/// 最后使用参数 data_field（ResponseDef.dataField）。
/// 返回 None 表示无 dataField 配置（调用方应直接使用原始 response）。
pub fn extract_data(
    response: &Value,
    envelope: Option<&Envelope>,
    data_field: Option<&str>,
) -> Result<Option<Value>, String> {
    // 优先级：接口级 envelope.dataField > 参数 data_field > 全局 envelope.dataField
    let field = envelope
        .and_then(|e| e.data_field.as_deref())
        .or(data_field);

    match field {
        Some(path) => {
            let val = jsonpath::extract(response, path).ok_or_else(|| {
                format!("响应数据字段 {} 不存在", path)
            })?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}

/// 解析有效 envelope（接口级优先，全局兜底）
fn resolve_envelope(envelope: Option<&Envelope>) -> Option<&Envelope> {
    envelope
}

/// 比较两个 JSON 值是否相等（支持类型宽松匹配）
fn values_equal(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::String(a), Value::Bool(b)) => a == "true" && *b || a == "false" && !*b,
        (Value::String(a), Value::Number(b)) => {
            a.parse::<f64>().ok() == b.as_f64()
        }
        (Value::Number(a), Value::String(b)) => {
            a.as_f64() == b.parse::<f64>().ok()
        }
        _ => actual == expected,
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(
            extract_error(&resp, Some(&e)),
            Some("未授权".to_string())
        );
    }

    #[test]
    fn test_extract_data() {
        let e = env("$.flag", json!(true), "$.msg", "$.data");
        let resp = json!({ "flag": true, "data": { "id": 1 } });
        let data = extract_data(&resp, Some(&e), None).unwrap();
        assert_eq!(data, Some(json!({ "id": 1 })));
    }
}
