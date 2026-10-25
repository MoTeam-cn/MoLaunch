//! 结构化 JSON 字符串收集与按 JSON Pointer 写回（规格 §2）

use std::collections::BTreeMap;

use serde_json::Value;

use super::super::json_value;

/// 收集全部字符串叶子：JSON Pointer → 文本
pub fn collect_structured_strings(content: &str) -> Result<BTreeMap<String, String>, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|e| format!("invalid structured JSON: {e}"))?;
    let mut entries = BTreeMap::new();
    collect(&value, String::new(), &mut entries);
    Ok(entries)
}

fn collect(value: &Value, pointer: String, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                collect(child, format!("{pointer}/{}", escape_segment(key)), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect(child, format!("{pointer}/{index}"), out);
            }
        }
        Value::String(text) => {
            out.insert(pointer, text.clone());
        }
        _ => {}
    }
}

/// Pointer 段转义：`~` → `~0`，`/` → `~1`
fn escape_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

/// 写回：保序解析后按 Pointer 替换，键序与数字写法保持不变
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
