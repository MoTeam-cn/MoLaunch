//! Slug 解析 + 中文名通配符处理

/// 解析 Slug 部分
/// 四种语法：
/// - `@slug` → CF=None, MR=slug
/// - `slug@` → CF=slug, MR=slug（相同）
/// - `cf_slug@mr_slug` → CF=cf_slug, MR=mr_slug
/// - `slug` → CF=slug, MR=None
pub(super) fn parse_slug_part(s: &str) -> (Option<String>, Option<String>) {
    if s.is_empty() {
        return (None, None);
    }
    if let Some(mr) = s.strip_prefix('@').map(|s| s.to_string()) {
        // @slug → 仅 Modrinth
        if mr.is_empty() {
            (None, None)
        } else {
            (None, Some(mr))
        }
    } else if let Some(slug) = s.strip_suffix('@').map(|s| s.to_string()) {
        // slug@ → CF=MR=slug
        if slug.is_empty() {
            (None, None)
        } else {
            (Some(slug.clone()), Some(slug))
        }
    } else if let Some(idx) = s.find('@') {
        // cf@mr → 双平台不同 slug
        let cf = s[..idx].to_string();
        let mr = s[idx + 1..].to_string();
        if cf.is_empty() && mr.is_empty() {
            (None, None)
        } else {
            (
                if cf.is_empty() { None } else { Some(cf) },
                if mr.is_empty() { None } else { Some(mr) },
            )
        }
    } else {
        // 无 @ → 仅 CurseForge
        (Some(s.to_string()), None)
    }
}

/// 处理中文名中的 * 通配符
/// * 替换为 " (Slug 去横线并首字母大写)"
pub(super) fn process_wildcard(name: &str, slug_part: &str) -> String {
    if !name.contains('*') {
        return name.to_string();
    }
    // 提取 slug 用于替换
    let slug = if let Some(idx) = slug_part.find('@') {
        let cf = &slug_part[..idx];
        let mr = &slug_part[idx + 1..];
        if !cf.is_empty() {
            cf
        } else {
            mr
        }
    } else if let Some(stripped) = slug_part.strip_prefix('@') {
        stripped
    } else if let Some(stripped) = slug_part.strip_suffix('@') {
        stripped
    } else {
        slug_part
    };
    // 去掉横线，首字母大写
    let replacement = capitalize(slug.replace('-', " "));
    name.replace('*', &format!(" ({})", replacement))
}

/// 首字母大写
pub(super) fn capitalize(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
