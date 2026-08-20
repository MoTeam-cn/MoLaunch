//! 结构化 JSON 字符串收集与按 JSON Pointer 写回

use std::collections::BTreeMap;

use serde_json::Value;

use super::super::json_value;

/// 收集结构化 JSON 的全部字符串叶子（JSON Pointer -> 文本）
pub fn collect_structured_strings(content: &str) -> Result<BTreeMap<String, String>, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|e| format!("invalid structured JSON: {e}"))?;
    let mut map = BTreeMap::new();
    walk_pointer(&value, "", &mut map);
    Ok(map)
}

fn walk_pointer(value: &Value, pointer: &str, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(obj) => {
            for (key, v) in obj {
                let p = if pointer.is_empty() {
                    format!("/{}", escape_pointer_segment(key))
                } else {
                    format!("{pointer}/{}", escape_pointer_segment(key))
                };
                walk_pointer(v, &p, out);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let p = format!("{pointer}/{i}");
                walk_pointer(v, &p, out);
            }
        }
        Value::String(s) => {
            out.insert(pointer.to_string(), s.clone());
        }
        _ => {}
    }
}

fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

/// 按 JSON Pointer 写回结构化 JSON：保序解析写回，键序/数字/布尔原样保留
pub fn apply_structured_strings(
    original: &str,
    translations: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut value = json_value::JsonValue::parse(original)
        .map_err(|e| format!("invalid structured JSON: {e}"))?;
    for (pointer, text) in translations {
        value
            .set_pointer(pointer, text.clone())
            .map_err(|e| format!("write structured JSON {pointer} failed: {e}"))?;
    }
    Ok(value.render_pretty())
}
