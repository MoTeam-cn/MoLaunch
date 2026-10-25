//! class 文本候选判定（规格 §3.4）
//!
//! 判定顺序与规格表格一致：已本地化 → 长度/字符无效 → 纯格式骨架 → 常量名 →
//! JVM 描述符 → 结构性字符串（URL / 路径 / 命名空间 / 小写标识符）→ 纯命令 →
//! 展示单词 → 自然语言。命中即返回，最后兜底为结构性字符串。
//!
//! 实现全部用字符扫描与手写描述符解析，不依赖正则，避免每次调用重复编译。

use super::super::types::has_chinese;

/// 判定结果：(是否候选, 分类标签)
pub fn classify_class_text(text: &str) -> (bool, &'static str) {
    let value = text.trim();
    if has_chinese(value) {
        return (false, "already_localized");
    }
    let length = value.chars().count();
    if !(3..=500).contains(&length) || !value.chars().any(|c| c.is_ascii_alphabetic()) {
        return (false, "invalid");
    }
    if value.contains('\0') {
        return (false, "invalid");
    }

    let plain = strip_legacy_codes(value);
    if plain.is_empty() {
        return (false, "invalid");
    }
    if is_format_skeleton(&plain) {
        return (false, "format_only");
    }
    if is_constant_name(&plain) {
        return (false, "constant");
    }
    if is_descriptor(&plain) {
        return (false, "descriptor");
    }
    if is_structural(&plain) {
        return (false, "structural");
    }
    if is_pure_command(&plain) {
        return (false, "pure_command");
    }
    if is_display_word(&plain) {
        return (true, "display_word");
    }
    if is_natural_language(&plain) {
        return (true, "natural_language");
    }
    (false, "structural")
}

/// 去掉 `§` + 颜色/格式码
fn strip_legacy_codes(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(current) = chars.next() {
        if current != '§' {
            out.push(current);
            continue;
        }
        if let Some(code) = chars.peek().copied() {
            if code.is_ascii_hexdigit() || matches!(code, 'k'..='o' | 'K'..='O' | 'r' | 'R') {
                chars.next();
                continue;
            }
        }
        out.push(current);
    }
    out.trim().to_string()
}

/// 剥掉占位符后只剩骨架（如 `%s %s`、`{0}`、`<name>`）
fn is_format_skeleton(value: &str) -> bool {
    let mut words: Vec<String> = Vec::new();
    let chars: Vec<char> = value.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let current = chars[index];
        if current == '%' {
            index += 1;
            while index < chars.len() && matches!(chars[index], '-' | '+' | ' ' | '0'..='9' | '.') {
                index += 1;
            }
            if index < chars.len() && chars[index].is_ascii_alphabetic() {
                index += 1;
            }
            continue;
        }
        if current == '{' || current == '<' {
            let closing = if current == '{' { '}' } else { '>' };
            let mut cursor = index + 1;
            let mut closed = false;
            while cursor < chars.len() {
                if chars[cursor] == closing {
                    closed = true;
                    break;
                }
                cursor += 1;
            }
            if closed {
                index = cursor + 1;
                continue;
            }
        }
        if current.is_ascii_alphabetic() {
            let mut word = String::new();
            while index < chars.len() && chars[index].is_ascii_alphabetic() {
                word.push(chars[index].to_ascii_lowercase());
                index += 1;
            }
            words.push(word);
            continue;
        }
        index += 1;
    }
    words.is_empty()
        || words
            .iter()
            .all(|word| matches!(word.as_str(), "x" | "y" | "z" | "c"))
}

/// 全大写 + 数字 + 下划线（常量名）
fn is_constant_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// JVM 字段/方法描述符（JVMS 4.3）
fn is_descriptor(value: &str) -> bool {
    let chars: Vec<char> = value.chars().collect();
    is_field_descriptor(&chars) || is_method_descriptor(&chars)
}

fn is_field_descriptor(chars: &[char]) -> bool {
    let mut index = 0;
    consume_field_type(chars, &mut index) && index == chars.len()
}

fn is_method_descriptor(chars: &[char]) -> bool {
    if chars.first() != Some(&'(') {
        return false;
    }
    let mut index = 1;
    while index < chars.len() && chars[index] != ')' {
        if !consume_field_type(chars, &mut index) {
            return false;
        }
    }
    if chars.get(index) != Some(&')') {
        return false;
    }
    index += 1;
    if chars.get(index) == Some(&'V') {
        return index + 1 == chars.len();
    }
    consume_field_type(chars, &mut index) && index == chars.len()
}

fn consume_field_type(chars: &[char], index: &mut usize) -> bool {
    while chars.get(*index) == Some(&'[') {
        *index += 1;
    }
    match chars.get(*index) {
        Some('B') | Some('C') | Some('D') | Some('F') | Some('I') | Some('J') | Some('S')
        | Some('Z') => {
            *index += 1;
            true
        }
        Some('L') => {
            *index += 1;
            let start = *index;
            while *index < chars.len() && chars[*index] != ';' {
                *index += 1;
            }
            *index > start && chars.get(*index) == Some(&';') && {
                *index += 1;
                true
            }
        }
        _ => false,
    }
}

/// 结构性字符串：URL / 路径 / 命名空间路径 / 小写标识符
fn is_structural(value: &str) -> bool {
    if value.contains(char::is_whitespace) {
        return false;
    }
    let lowered = value.to_ascii_lowercase();
    if ["http://", "https://", "ftp://"]
        .iter()
        .any(|prefix| lowered.starts_with(prefix))
    {
        return true;
    }
    if is_windows_path(value) || is_unix_path(value) || is_namespaced_path(value) {
        return true;
    }
    value.chars().all(|c| {
        c.is_ascii_lowercase()
            || c.is_ascii_digit()
            || matches!(c, '_' | '$' | '.' | ':' | '/' | '-')
    })
}

fn is_windows_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    if value.starts_with("\\\\") {
        return true;
    }
    bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\'
}

fn is_unix_path(value: &str) -> bool {
    if !value.starts_with('/') || value.contains(char::is_whitespace) {
        return false;
    }
    value.trim_matches('/').contains('/') || value.matches('/').count() >= 2
}

/// `ns:path` / `a/b/c` 形态：以分隔符收尾的段 + 末尾标识符
fn is_namespaced_path(value: &str) -> bool {
    let separators = value.matches(['/', ':']).count();
    if separators == 0 {
        return false;
    }
    let tail = value.rsplit(['/', ':']).next().unwrap_or_default();
    let tail_ok = !tail.is_empty()
        && tail
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '.' | '-'));
    let head_ok = value
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        .unwrap_or(false);
    tail_ok && head_ok
}

/// 纯命令：`/cmd arg arg`
fn is_pure_command(value: &str) -> bool {
    if !value.starts_with('/') {
        return false;
    }
    if value.contains(" - ") || value.contains(" — ") || value.contains(" : ") {
        return false;
    }
    let mut tokens = value.split_whitespace();
    let Some(head) = tokens.next() else {
        return false;
    };
    let command = head.trim_start_matches('/');
    let head_ok = !command.is_empty()
        && command
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | ':' | '-'));
    head_ok && tokens.all(is_command_argument)
}

fn is_command_argument(token: &str) -> bool {
    // 命令参数常见形态：普通字面量、选择器（@p / @e[type=x]）、坐标（~ ~1 ~）、通配符
    let plain = |c: char| {
        c.is_ascii_alphanumeric()
            || matches!(c, '_' | '.' | ':' | '-' | '@' | '~' | '^' | '*' | '!')
    };
    if token.chars().all(plain) {
        return true;
    }
    for (open, close) in [('{', '}'), ('<', '>'), ('[', ']')] {
        if let Some(inner) = token
            .strip_prefix(open)
            .and_then(|rest| rest.strip_suffix(close))
        {
            return !inner.is_empty()
                && !inner.contains(char::is_whitespace)
                && !inner.contains(open)
                && !inner.contains(close);
        }
    }
    false
}

/// 单个首字母大写的展示词
fn is_display_word(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_uppercase() => {}
        _ => return false,
    }
    let rest = chars.as_str();
    !rest.is_empty()
        && rest
            .chars()
            .all(|c| c.is_ascii_alphabetic() || matches!(c, '\'' | '-'))
}

/// 含空格或标点
fn is_natural_language(value: &str) -> bool {
    value.contains(char::is_whitespace)
        || value
            .chars()
            .any(|c| matches!(c, '!' | '?' | '.' | ',' | ':' | '\'' | '"' | '-'))
}
