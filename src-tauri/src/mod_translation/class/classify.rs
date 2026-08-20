//! class 文本候选判定与格式骨架识别

use super::super::types::has_chinese;

/// 候选判定：返回 (是否候选, 分类)
pub fn classify_class_text(text: &str) -> (bool, &'static str) {
    let value = text.trim();
    if has_chinese(value) {
        return (false, "already_localized");
    }
    if value.chars().count() < 3
        || value.chars().count() > 500
        || !value.chars().any(|c| c.is_ascii_alphabetic())
        || value.contains('\0')
    {
        return (false, "invalid");
    }
    let plain = regex::Regex::new(r"§[0-9A-FK-OR]")
        .unwrap()
        .replace_all(value, "")
        .trim()
        .to_string();
    if is_format_only(&plain) {
        return (false, "format_only");
    }
    if regex::Regex::new(r"^[A-Z0-9_]+$").unwrap().is_match(&plain) {
        return (false, "constant");
    }
    let descriptor = regex::Regex::new(
        r"^(?:\[*[BCDFIJSVZ]|\[*L[\w/$]+;|\((?:\[*[BCDFIJSVZ]|\[*L[\w/$]+;)*\)(?:\[*[BCDFIJSVZ]|\[*L[\w/$]+;))$",
    )
    .unwrap();
    if descriptor.is_match(&plain) {
        return (false, "descriptor");
    }
    let pure_url = regex::Regex::new(r"^(?:https?|ftp)://\S+$")
        .unwrap()
        .is_match(&plain);
    let pure_windows_path = regex::Regex::new(r"^(?:[A-Za-z]:\\|\\\\)[^\r\n]+$")
        .unwrap()
        .is_match(&plain);
    let pure_unix_path = regex::Regex::new(r"^/(?:[^\s/]+/)+[^\s/]*$")
        .unwrap()
        .is_match(&plain);
    let pure_internal_path = regex::Regex::new(r"^(?:[A-Za-z_$][\w$.-]*[/:]){1,}[A-Za-z0-9_$.-]+$")
        .unwrap()
        .is_match(&plain);
    let pure_lower_identifier = regex::Regex::new(r"^[a-z0-9_$.:/-]+$")
        .unwrap()
        .is_match(&plain);
    if pure_url
        || pure_windows_path
        || pure_unix_path
        || pure_internal_path
        || pure_lower_identifier
    {
        return (false, "structural");
    }
    let pure_command = regex::Regex::new(
        r"^/[A-Za-z0-9_.:-]+(?:\s+(?:[A-Za-z0-9_.:-]+|\{[^{}\s]+\}|<[^<>\s]+>|\[[^\[\]\s]+\]))*$",
    )
    .unwrap();
    if !regex::Regex::new(r"\s(?:-|—|:)\s")
        .unwrap()
        .is_match(&plain)
        && pure_command.is_match(&plain)
    {
        return (false, "pure_command");
    }
    if regex::Regex::new(r"^[A-Z][A-Za-z'-]+$")
        .unwrap()
        .is_match(&plain)
    {
        return (true, "display_word");
    }
    if regex::Regex::new(r#"\s|[!?.,:'"-]"#)
        .unwrap()
        .is_match(&plain)
    {
        return (true, "natural_language");
    }
    (false, "structural")
}

/// 剥占位符后是否仅剩格式骨架
fn is_format_only(text: &str) -> bool {
    let remainder = regex::Regex::new(r"%[-+ 0-9.]*[a-zA-Z]|\{[^{}]*\}|<[^<>]*>")
        .unwrap()
        .replace_all(text, " ")
        .to_string();
    let words = regex::Regex::new(r"[A-Za-z]+")
        .unwrap()
        .find_iter(&remainder)
        .map(|m| m.as_str().to_ascii_lowercase())
        .collect::<Vec<_>>();
    words.is_empty()
        || words
            .iter()
            .all(|w| matches!(w.as_str(), "x" | "y" | "z" | "c"))
}
