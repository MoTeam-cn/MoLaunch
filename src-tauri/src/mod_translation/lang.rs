//! 模组翻译：语言文件读写（JSON / key-value / 结构化 JSON / 自由文本）

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use super::json_value;

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

/// 解析 key-value 语言文件（.lang / .properties），保留注释与空白行供写回
pub fn parse_keyvalue(content: &str) -> (Vec<(String, String)>, Vec<String>) {
    // lines: 原文逐行（注释/空白/键值），kv: 可翻译键值对
    let mut kv: Vec<(String, String)> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }
        // 分隔符取第一个未转义的 =/:，转义分隔符（\= 或 \:）视为键的一部分
        if let Some(sep) = find_unescaped_separator(line) {
            let key = line[..sep].trim().to_string();
            let value = line[sep + 1..].trim().to_string();
            if !key.is_empty() {
                kv.push((key, value));
            }
        }
    }
    (kv, content.lines().map(|l| l.to_string()).collect())
}

/// 查找第一个未被反斜杠转义的分隔符（= 或 :）
fn find_unescaped_separator(line: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, ch) in line.char_indices() {
        if ch == '\\' {
            escaped = !escaped;
            continue;
        }
        if (ch == '=' || ch == ':') && !escaped {
            return Some(i);
        }
        escaped = false;
    }
    None
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
            let (key, eq_pos, sep_len) = if let Some(sep) = find_unescaped_separator(line) {
                let sep_char = line[sep..].chars().next().unwrap_or('=');
                (line[..sep].trim(), Some(sep), sep_char.len_utf8())
            } else {
                (line.trim(), None, 0)
            };
            if let Some(eq_pos) = eq_pos {
                if let Some(t) = translations.get(key) {
                    // 保留原分隔符与缩进：重建为 "key=value"
                    let indent = line.len() - line.trim_start().len();
                    let prefix = if indent > 0 {
                        " ".repeat(indent)
                    } else {
                        String::new()
                    };
                    let sep = line[eq_pos..eq_pos + sep_len].to_string();
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

/// 自由文本文件布局快照：BOM / EOL / 尾换行 / 源行 / 已有目标行
#[derive(Debug, Clone)]
pub struct FreeTextSnapshot {
    pub has_bom: bool,
    pub eol: String,
    pub trailing_newline: bool,
    pub source_lines: Vec<String>,
    pub target_lines: Vec<String>,
}

/// 解析自由文本：检测 BOM、EOL（优先 \r\n）、尾换行，按 EOL 分行
pub fn snapshot_free_text(content: &str) -> FreeTextSnapshot {
    let body = content.strip_prefix('\u{feff}').unwrap_or(content);
    let has_bom = body.len() != content.len();
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let trailing_newline = body.ends_with("\r\n") || body.ends_with('\n');
    let mut source_lines: Vec<String> = body
        .split("\r\n")
        .flat_map(|part| part.split('\n'))
        .map(str::to_string)
        .collect();
    if trailing_newline {
        source_lines.pop();
    }
    FreeTextSnapshot {
        has_bom,
        eol: eol.to_string(),
        trailing_newline,
        source_lines,
        target_lines: Vec::new(),
    }
}

/// 按布局重建文本：target_lines 为基底，空（未翻译）行回退 source_lines
pub fn render_localized_text(snapshot: &FreeTextSnapshot) -> String {
    let mut out = String::new();
    if snapshot.has_bom {
        out.push('\u{feff}');
    }
    let count = snapshot.source_lines.len().max(snapshot.target_lines.len());
    for i in 0..count {
        if i > 0 {
            out.push_str(&snapshot.eol);
        }
        let text = snapshot
            .target_lines
            .get(i)
            .filter(|l| !l.trim().is_empty())
            .or_else(|| snapshot.source_lines.get(i))
            .map(String::as_str)
            .unwrap_or("");
        out.push_str(text);
    }
    if snapshot.trailing_newline && count > 0 {
        out.push_str(&snapshot.eol);
    }
    out
}

/// 行号 -> /lines/000000 形式键
fn line_key(index: usize) -> String {
    format!("/lines/{index:06}")
}

/// 解析 /lines/NNNNNN 形式键（兼容纯行号字符串）
fn parse_line_key(key: &str) -> Option<usize> {
    key.strip_prefix("/lines/")
        .or(Some(key))
        .and_then(|s| s.parse::<usize>().ok())
}

/// 读取已有 zh_cn 自由文本（存在时）按行号解析为 /lines/NNNNNN 键，供断点续传合并
pub fn read_localized_target(workspace: &Path, target_path: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let Ok(content) = std::fs::read_to_string(workspace.join(target_path)) else {
        return map;
    };
    for (i, line) in snapshot_free_text(&content).source_lines.iter().enumerate() {
        if !line.trim().is_empty() {
            map.insert(line_key(i), line.clone());
        }
    }
    map
}

/// 自由文本按行对齐：仅替换非空行（跳过空翻译与空源行），保持行数
pub fn align_free_text(lines: &[String], translations: &BTreeMap<String, String>) -> Vec<String> {
    let mut out: Vec<String> = lines.to_vec();
    for (key, text) in translations {
        let Some(index) = parse_line_key(key) else {
            continue;
        };
        if text.trim().is_empty() || index >= out.len() || out[index].trim().is_empty() {
            continue;
        }
        out[index] = text.clone();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_json_keeps_key_order_and_scalars() {
        let original = "{\"z\": \"Hello\", \"a\": {\"n\": 10, \"b\": true, \"title\": \"Hi\"}}";
        let translations = BTreeMap::from([
            ("/z".to_string(), "你好".to_string()),
            ("/a/title".to_string(), "标题".to_string()),
        ]);
        let out = apply_structured_strings(original, &translations).unwrap();
        assert!(
            out.find("\"z\"").unwrap() < out.find("\"a\"").unwrap(),
            "{out}"
        );
        assert!(
            out.contains("\"n\": 10") && out.contains("\"b\": true"),
            "{out}"
        );
        assert!(out.contains("\"z\": \"你好\"") && out.contains("\"title\": \"标题\""));
    }

    #[test]
    fn free_text_snapshot_round_trips_bom_eol() {
        let content = "\u{feff}line1\r\nline2\r\n";
        let snap = snapshot_free_text(content);
        assert!(snap.has_bom);
        assert_eq!(snap.eol, "\r\n");
        assert!(snap.trailing_newline);
        assert_eq!(snap.source_lines, vec!["line1", "line2"]);
        assert_eq!(render_localized_text(&snap), content);
    }

    #[test]
    fn render_localized_text_falls_back_to_source() {
        let mut snap = snapshot_free_text("a\nb\nc\n");
        snap.target_lines = vec!["甲".to_string(), String::new()];
        assert_eq!(render_localized_text(&snap), "甲\nb\nc\n");
    }

    #[test]
    fn read_localized_target_maps_lines_to_keys() {
        let dir = std::env::temp_dir();
        let target = dir.join("mo_launch_test_zh_cn.txt");
        std::fs::write(&target, "\u{feff}甲\n\n乙\n").unwrap();
        let map = read_localized_target(&dir, &target);
        std::fs::remove_file(&target).ok();
        assert_eq!(map.get("/lines/000000").map(String::as_str), Some("甲"));
        assert_eq!(map.get("/lines/000002").map(String::as_str), Some("乙"));
        assert!(!map.contains_key("/lines/000001"));
    }

    #[test]
    fn parse_keyvalue_handles_escaped_separators() {
        let content = "a=1\nb:two\nc\\=x=3\nd\\:y=4\n";
        let (kv, _) = parse_keyvalue(content);
        let map: BTreeMap<String, String> = kv.into_iter().collect();
        assert_eq!(map.get("a").map(String::as_str), Some("1"));
        assert_eq!(map.get("b").map(String::as_str), Some("two"));
        assert_eq!(map.get("c\\=x").map(String::as_str), Some("3"));
        assert_eq!(map.get("d\\:y").map(String::as_str), Some("4"));
    }
}
