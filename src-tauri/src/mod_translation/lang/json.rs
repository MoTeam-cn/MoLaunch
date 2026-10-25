//! 标准 JSON 语言文件读写（规格 §2）
//!
//! 读取时把嵌套对象摊平成 `点分路径 → 字符串`，写回时按同样路径回填，
//! 数字/布尔/数组等非字符串叶子保持原样。

use std::collections::BTreeMap;

use serde_json::Value;

/// 拼接点分路径
fn join(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

/// 读取：仅收集字符串叶子
pub fn read_json_lang(content: &str) -> Result<BTreeMap<String, String>, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|e| format!("invalid JSON language file: {e}"))?;
    let mut entries = BTreeMap::new();
    collect(&value, "", &mut entries);
    Ok(entries)
}

fn collect(value: &Value, prefix: &str, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                collect(child, &join(prefix, key), out);
            }
        }
        Value::String(text) => {
            out.insert(prefix.to_string(), text.clone());
        }
        _ => {}
    }
}

/// 写回：只替换命中路径的字符串叶子
pub fn write_json_lang(
    original: &str,
    translations: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut value: Value =
        serde_json::from_str(original).map_err(|e| format!("invalid JSON language file: {e}"))?;
    apply(&mut value, "", translations);
    serde_json::to_string_pretty(&value).map_err(|e| format!("serialize zh_cn.json failed: {e}"))
}

fn apply(value: &mut Value, prefix: &str, translations: &BTreeMap<String, String>) {
    let Value::Object(map) = value else {
        return;
    };
    for (key, child) in map.iter_mut() {
        let path = join(prefix, key);
        if let Value::String(text) = child {
            if let Some(translated) = translations.get(&path) {
                *text = translated.clone();
                continue;
            }
        }
        apply(child, &path, translations);
    }
}
