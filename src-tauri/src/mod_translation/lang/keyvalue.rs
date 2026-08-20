//! key-value 语言文件解析与写回（.lang / .properties）

use std::collections::BTreeMap;

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
