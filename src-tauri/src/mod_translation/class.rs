//! 模组翻译：class 常量池解析、候选发现与 UTF8 改写

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::jar;
use super::types::{has_chinese, ClassCandidate};

/// 解析 class 常量池，返回全部 Utf8 条目 (文本, 起始偏移, 结束偏移)
pub fn parse_class_constant_pool(bytes: &[u8]) -> Result<Vec<(String, usize, usize)>, String> {
    Ok(parse_pool(bytes)?
        .0
        .into_iter()
        .map(|(_, text, start, end)| (text, start, end))
        .collect())
}

/// 被 String 常量引用的 Utf8 文本（运行时可见字符串）
pub fn class_string_constants(bytes: &[u8]) -> Vec<String> {
    let Ok((entries, string_indices)) = parse_pool(bytes) else {
        return Vec::new();
    };
    entries
        .into_iter()
        .filter(|(index, _, _, _)| string_indices.contains(index))
        .map(|(_, text, _, _)| text)
        .collect()
}

/// 常量池解析结果：Utf8 条目(索引,文本,start,end) + String 引用的 Utf8 索引
type PoolEntries = (Vec<(u16, String, usize, usize)>, HashSet<u16>);

/// 解析常量池
fn parse_pool(bytes: &[u8]) -> Result<PoolEntries, String> {
    if bytes.len() < 10 || bytes[0..4] != [0xca, 0xfe, 0xba, 0xbe] {
        return Err("not a valid Java class file".to_string());
    }
    let count = u16::from_be_bytes([bytes[8], bytes[9]]);
    let mut entries = Vec::new();
    let mut string_indices = HashSet::new();
    let mut offset = 10usize;
    let mut index = 1u16;
    while index < count {
        if offset >= bytes.len() {
            return Err("class constant pool is truncated".to_string());
        }
        let tag = bytes[offset];
        offset += 1;
        match tag {
            1 => {
                if offset + 2 > bytes.len() {
                    return Err("class UTF-8 constant is truncated".to_string());
                }
                let length = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
                offset += 2;
                if offset + length > bytes.len() {
                    return Err("class UTF-8 constant is truncated".to_string());
                }
                let text = String::from_utf8_lossy(&bytes[offset..offset + length]).into_owned();
                entries.push((index, text, offset - 3, offset + length));
                offset += length;
            }
            3 | 4 => offset += 4,
            5 | 6 => {
                offset += 8;
                index += 1;
            }
            8 => {
                if offset + 2 > bytes.len() {
                    return Err("class String constant is truncated".to_string());
                }
                string_indices.insert(u16::from_be_bytes([bytes[offset], bytes[offset + 1]]));
                offset += 2;
            }
            7 | 16 | 19 | 20 => offset += 2,
            9 | 10 | 11 | 12 | 17 | 18 => offset += 4,
            15 => offset += 3,
            _ => return Err(format!("unsupported class constant pool tag: {tag}")),
        }
        if offset > bytes.len() {
            return Err("class constant pool is truncated".to_string());
        }
        index += 1;
    }
    Ok((entries, string_indices))
}

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

/// 遍历工作区 .class 文件，按文本跨文件聚合候选
pub fn discover_class_candidates(workspace: &Path) -> Vec<ClassCandidate> {
    let mut grouped: HashMap<String, (String, BTreeSet<String>)> = HashMap::new();
    for relative in jar::collect_files(workspace).unwrap_or_default() {
        if !relative.to_ascii_lowercase().ends_with(".class") {
            continue;
        }
        let Ok(bytes) = std::fs::read(workspace.join(&relative)) else {
            continue;
        };
        for text in class_string_constants(&bytes) {
            if !classify_class_text(&text).0 {
                continue;
            }
            let entry = grouped
                .entry(text.clone())
                .or_insert_with(|| (relative.clone(), BTreeSet::new()));
            entry.1.insert(relative.clone());
        }
    }
    let mut candidates: Vec<ClassCandidate> = grouped
        .into_iter()
        .map(|(text, (path, paths))| {
            let paths = paths.into_iter().collect::<Vec<_>>();
            ClassCandidate {
                id: candidate_id(&path, &text),
                path: path.clone(),
                paths: paths.clone(),
                occurrences: paths.len(),
                text,
            }
        })
        .collect();
    candidates.sort_by(|a, b| a.id.cmp(&b.id));
    candidates
}

fn candidate_id(path: &str, text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    hasher.update(b"\0");
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())[..24].to_string()
}

/// 只重建被替换 Utf8 条目的字节区间，其余原样拷贝
pub fn replace_class_utf8(bytes: &[u8], text: &str, replacement: &str) -> Result<Vec<u8>, String> {
    let (entries, _) = parse_pool(bytes)?;
    let encoded = replacement.as_bytes();
    if encoded.len() > 65_535 {
        return Err("replacement class text is too long".to_string());
    }
    let mut out = Vec::with_capacity(bytes.len());
    let mut cursor = 0usize;
    let mut changed = 0usize;
    for (_, entry_text, start, end) in &entries {
        if entry_text != text || replacement == text {
            continue;
        }
        out.extend_from_slice(&bytes[cursor..*start]);
        out.push(1);
        out.extend_from_slice(&(encoded.len() as u16).to_be_bytes());
        out.extend_from_slice(encoded);
        cursor = *end;
        changed += 1;
    }
    if changed == 0 {
        return Ok(bytes.to_vec());
    }
    out.extend_from_slice(&bytes[cursor..]);
    Ok(out)
}

/// 确定性排除理由：java 类名 / 正则 / 内部诊断；UI 宿主路径保留
pub fn deterministic_class_exclusion_reason(path: &str, text: &str) -> Option<&'static str> {
    let value = text.trim();
    if regex::Regex::new(r"^(?:[a-z_][A-Za-z0-9_$]*\.)+[A-Z_$][A-Za-z0-9_$]*$")
        .unwrap()
        .is_match(value)
    {
        return Some("java_class_name");
    }
    let looks_like_regex = value.starts_with('^')
        && value.ends_with('$')
        && regex::Regex::new(r"(?:\\[.dDsSwW]|\[[^\]]+\]|\{\d+(?:,\d*)?\}|\(\?:?)")
            .unwrap()
            .is_match(value);
    if looks_like_regex {
        return Some("regular_expression");
    }
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    if regex::Regex::new(r"(?:^|/)(?:gui|screen|widget|tooltip|chat|menu|config)(?:/|$)")
        .unwrap()
        .is_match(&normalized)
    {
        return None;
    }
    let diagnostic_host = regex::Regex::new(
        r"(?:^|/)(?:file|graphics|server|palette|world|misc|region|pool|core)(?:/|$)",
    )
    .unwrap()
    .is_match(&normalized);
    let diagnostic_text =
        regex::Regex::new(r"(?i)\b(?:ioexception|failed to|retrying|unknown status)\b")
            .unwrap()
            .is_match(value);
    if diagnostic_host && diagnostic_text {
        return Some("internal_diagnostic");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 最小合法 class 文件：3 个 Utf8 + 2 个 Class + 1 个 String 引用
    fn fixture() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 0x3d]);
        bytes.extend_from_slice(&[0, 7]); // constant_pool_count = 7（索引 1-6）
        for value in ["Hello", "World", "Iron Ingot"] {
            bytes.push(1);
            bytes.extend_from_slice(&(value.len() as u16).to_be_bytes());
            bytes.extend_from_slice(value.as_bytes());
        }
        bytes.extend_from_slice(&[7, 0, 1]); // Class -> 1
        bytes.extend_from_slice(&[7, 0, 2]); // Class -> 2
        bytes.extend_from_slice(&[8, 0, 3]); // String -> 3
        bytes.extend_from_slice(&[0, 1]); // access_flags
        bytes.extend_from_slice(&[0, 4]); // this_class
        bytes.extend_from_slice(&[0, 5]); // super_class
        bytes.extend_from_slice(&[0, 0]); // interfaces_count
        bytes.extend_from_slice(&[0, 0]); // fields_count
        bytes.extend_from_slice(&[0, 0]); // methods_count
        bytes.extend_from_slice(&[0, 0]); // attributes_count
        bytes
    }

    #[test]
    fn parses_constant_pool_and_string_refs() {
        let bytes = fixture();
        let entries = parse_class_constant_pool(&bytes).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].0, "Hello");
        assert_eq!(entries[2].0, "Iron Ingot");
        assert_eq!(class_string_constants(&bytes), vec!["Iron Ingot"]);
    }

    #[test]
    fn classifies_candidate_text() {
        assert_eq!(
            classify_class_text("Iron Ingot"),
            (true, "natural_language")
        );
        assert_eq!(
            classify_class_text("Welcome to the server"),
            (true, "natural_language")
        );
        assert_eq!(classify_class_text("IronIngot"), (true, "display_word"));
        assert_eq!(
            classify_class_text("minecraft:iron_ingot"),
            (false, "structural")
        );
        assert_eq!(classify_class_text("ITEM_REGISTRY"), (false, "constant"));
        assert_eq!(classify_class_text("已翻译"), (false, "already_localized"));
        assert_eq!(classify_class_text("%s: %s"), (false, "format_only"));
    }

    #[test]
    fn replaces_utf8_and_preserves_structure() {
        let bytes = fixture();
        let rewritten = replace_class_utf8(&bytes, "Iron Ingot", "铁锭").unwrap();
        assert!(rewritten.starts_with(&[0xca, 0xfe, 0xba, 0xbe]));
        let entries = parse_class_constant_pool(&rewritten).unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().any(|(t, _, _)| t == "铁锭"));
        assert_eq!(class_string_constants(&rewritten), vec!["铁锭"]);
        assert_eq!(replace_class_utf8(&bytes, "Nope", "x").unwrap(), bytes);
    }

    #[test]
    fn aggregates_candidates_across_files() {
        let dir = std::env::temp_dir().join(format!("mt-class-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let bytes = fixture();
        std::fs::write(dir.join("a.class"), &bytes).unwrap();
        std::fs::write(dir.join("b.class"), &bytes).unwrap();
        let candidates = discover_class_candidates(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].text, "Iron Ingot");
        assert_eq!(candidates[0].occurrences, 2);
        assert_eq!(candidates[0].paths.len(), 2);
        assert_eq!(candidates[0].id.len(), 24);
    }

    #[test]
    fn deterministic_exclusions() {
        assert_eq!(
            deterministic_class_exclusion_reason(
                "xaero/map/file/MapProcessor.class",
                "IOException trying to detect map files!"
            ),
            Some("internal_diagnostic")
        );
        assert_eq!(
            deterministic_class_exclusion_reason(
                "xaero/map/gui/GuiMap.class",
                "Failed to load your map. Retry?"
            ),
            None
        );
        assert_eq!(
            deterministic_class_exclusion_reason("a/b/SomeClass.class", "com.example.SomeClass"),
            Some("java_class_name")
        );
    }
}
