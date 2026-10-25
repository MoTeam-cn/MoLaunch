//! key-value 语言文件解析与写回（.lang / .properties，规格 §2）
//!
//! 分隔符取第一个未被反斜杠转义的 `=` 或 `:`；写回时保留缩进、分隔符与
//! 注释/空行，只替换命中键的值。

use std::collections::BTreeMap;

/// 一行键值：缩进长度、键、分隔符、值
struct Entry<'a> {
    indent: usize,
    key: &'a str,
    separator: char,
    value: &'a str,
}

/// 定位第一个未转义的分隔符
fn separator_index(line: &str) -> Option<usize> {
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        match ch {
            '\\' => escaped = !escaped,
            '=' | ':' if !escaped => return Some(index),
            _ => escaped = false,
        }
    }
    None
}

fn is_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!')
}

fn parse_entry(line: &str) -> Option<Entry<'_>> {
    if is_comment(line) {
        return None;
    }
    let index = separator_index(line)?;
    let indent = line.len() - line.trim_start().len();
    let key = line[..index].trim();
    if key.is_empty() {
        return None;
    }
    let separator = line[index..].chars().next().unwrap_or('=');
    Some(Entry {
        indent,
        key,
        separator,
        value: line[index + 1..].trim(),
    })
}

/// 解析：返回可翻译键值对与原文行（写回用）
pub fn parse_keyvalue(content: &str) -> (Vec<(String, String)>, Vec<String>) {
    let lines: Vec<String> = content.lines().map(str::to_string).collect();
    let entries = lines
        .iter()
        .filter_map(|line| parse_entry(line))
        .map(|entry| (entry.key.to_string(), entry.value.to_string()))
        .collect();
    (entries, lines)
}

/// 写回：逐行替换命中键，保留原有缩进与分隔符
pub fn write_keyvalue(
    original_lines: &[String],
    translations: &BTreeMap<String, String>,
) -> String {
    let mut out = String::new();
    for line in original_lines {
        match parse_entry(line) {
            Some(entry) => match translations.get(entry.key) {
                Some(translated) => {
                    out.push_str(&" ".repeat(entry.indent));
                    out.push_str(entry.key);
                    out.push(entry.separator);
                    out.push_str(translated);
                }
                None => out.push_str(line),
            },
            None => out.push_str(line),
        }
        out.push('\n');
    }
    out
}
