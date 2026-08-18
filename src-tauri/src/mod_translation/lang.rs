//! 模组翻译：语言文件读写（JSON / key-value / 结构化 JSON / 自由文本）

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
    match value {
        Value::Object(obj) => {
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
        _ => {}
    }
}

/// 解析 key-value 语言文件（.lang / .properties），保留注释与空白行供写回
pub fn parse_keyvalue(content: &str) -> (Vec<(String, String)>, Vec<String>) {
    // lines: 原文逐行（注释/空白/键值），kv: 可翻译键值对
    let mut kv: Vec<(String, String)> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim().to_string();
            let value = line[eq + 1..].trim().to_string();
            if !key.is_empty() {
                kv.push((key, value));
            }
        } else if let Some(colon) = line.find(':') {
            let key = line[..colon].trim().to_string();
            let value = line[colon + 1..].trim().to_string();
            if !key.is_empty() {
                kv.push((key, value));
            }
        }
    }
    (kv, content.lines().map(|l| l.to_string()).collect())
}

/// 写回 key-value 语言文件：逐行替换已翻译值，保留注释/EOL 结构
pub fn write_keyvalue(
    original_lines: &[String],
    translations: &BTreeMap<String, String>,
) -> String {
    let mut out = String::new();
    for line in original_lines {
        let trimmed = line.trim_start();
        let replaced = if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!')
        {
            false
        } else {
            let (key, eq_pos, sep_len) = if let Some(eq) = line.find('=') {
                (line[..eq].trim(), Some(eq), 1)
            } else if let Some(colon) = line.find(':') {
                (line[..colon].trim(), Some(colon), 1)
            } else {
                (line.trim(), None, 0)
            };
            if eq_pos.is_some() {
                if let Some(t) = translations.get(key) {
                    // 保留原分隔符与缩进：重建为 "key=value"
                    let indent = line.len() - line.trim_start().len();
                    let prefix = if indent > 0 {
                        " ".repeat(indent)
                    } else {
                        String::new()
                    };
                    let sep = line[eq_pos.unwrap()..eq_pos.unwrap() + sep_len].to_string();
                    out.push_str(&format!("{prefix}{key}{sep}{t}"));
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if !replaced {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

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

/// 按 JSON Pointer 写回结构化 JSON（替换字符串叶子）
pub fn apply_structured_strings(
    original: &str,
    translations: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut value: Value =
        serde_json::from_str(original).map_err(|e| format!("invalid structured JSON: {e}"))?;
    for (pointer, text) in translations {
        if let Some(target) = pointer_value_mut(&mut value, pointer) {
            *target = Value::String(text.clone());
        }
    }
    serde_json::to_string_pretty(&value)
        .map_err(|e| format!("serialize structured JSON failed: {e}"))
}

fn pointer_value_mut<'a>(value: &'a mut Value, pointer: &str) -> Option<&'a mut Value> {
    let mut current = value;
    for raw in pointer.trim_start_matches('/').split('/') {
        let segment = raw.replace("~1", "/").replace("~0", "~");
        current = match current {
            Value::Object(obj) => obj.get_mut(&segment)?,
            Value::Array(arr) => arr.get_mut(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

/// 自由文本按行对齐：仅替换非空、非纯占位符行，保持行数与空行结构
pub fn align_free_text(lines: &[String], translations: &BTreeMap<String, String>) -> Vec<String> {
    let mut out: Vec<String> = lines.to_vec();
    for (index, text) in translations {
        if let Ok(i) = index.parse::<usize>() {
            if i < out.len() {
                out[i] = text.clone();
            }
        }
    }
    out
}
