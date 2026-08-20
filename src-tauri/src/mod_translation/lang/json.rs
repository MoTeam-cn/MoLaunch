//! 标准 JSON 语言文件读写（扁平映射，保留原结构）

use std::collections::BTreeMap;

use serde_json::Value;

/// 读取标准 JSON 语言文件为扁平映射（仅字符串值可译；其余原样保留）
pub fn read_json_lang(content: &str) -> Result<BTreeMap<String, String>, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|e| format!("invalid JSON language file: {e}"))?;
    let mut map = BTreeMap::new();
    collect_flat_strings(&value, "", &mut map);
    Ok(map)
}

fn collect_flat_strings(value: &Value, prefix: &str, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(obj) => {
            for (key, v) in obj {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_flat_strings(v, &path, out);
            }
        }
        Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        _ => {}
    }
}

/// 写回标准 JSON 语言文件：保留原结构，仅替换已翻译的字符串叶子
pub fn write_json_lang(
    original: &str,
    translations: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut value: Value =
        serde_json::from_str(original).map_err(|e| format!("invalid JSON language file: {e}"))?;
    apply_flat_translations(&mut value, "", translations);
    serde_json::to_string_pretty(&value).map_err(|e| format!("serialize zh_cn.json failed: {e}"))
}

fn apply_flat_translations(
    value: &mut Value,
    prefix: &str,
    translations: &BTreeMap<String, String>,
) {
    if let Value::Object(obj) = value {
        for (key, v) in obj.iter_mut() {
            let path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            if let Some(t) = translations.get(&path) {
                if let Value::String(s) = v {
                    *s = t.clone();
                    continue;
                }
            }
            apply_flat_translations(v, &path, translations);
        }
    }
}
