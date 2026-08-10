//! 版本号提取工具
//!
//! 从文件名或显示名中提取版本号，用于 CurseForge 等不直接提供版本号的平台。
//! 例：`jei-1.20.1-15.2.0.27.jar` / `alltheleaks-1.1.1+1.20.1-forge.jar` → 提取 mod 版本号（非 MC 版本号）

/// 扫描字符串中的 `数字.数字[.数字]*` 标记
fn find_version_tokens(name: &str) -> Vec<String> {
    let chars: Vec<char> = name.chars().collect();
    let mut matches: Vec<String> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            let mut end = i + 1;
            while end < chars.len() {
                if chars[end].is_ascii_digit() {
                    end += 1;
                } else if chars[end] == '.'
                    && end + 1 < chars.len()
                    && chars[end + 1].is_ascii_digit()
                {
                    end += 2;
                } else {
                    break;
                }
            }
            if end > start + 1 {
                let ver: String = chars[start..end].iter().collect();
                if ver.contains('.') {
                    matches.push(ver);
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
    matches
}

/// 从文件名或显示名提取版本号
///
/// 提取策略：
/// 1. 去掉扩展名（.jar / .disabled / .old / .litemod）
/// 2. 如果有 `+` 分隔符（如 `modname-1.1.1+1.20.1`），取 `+` 前面的部分，
///    在该部分中找第一个 `数字.数字[.数字]*` 模式
/// 3. 如果没有 `+`，匹配所有 `数字.数字[.数字]*` 模式，取最后一个
///    （通常 mod 版本号在 MC 版本号后面，如 `create-1.20.1-6.0.4` → `6.0.4`）
pub fn extract_version_from_name(name: &str) -> String {
    let name = name
        .trim_end_matches(".jar")
        .trim_end_matches(".disabled")
        .trim_end_matches(".old")
        .trim_end_matches(".litemod");

    let matches = find_version_tokens(name);
    if matches.is_empty() {
        return String::new();
    }
    // 有 `+` 分隔符时取 `+` 前面的第一个版本号
    // （如 `alltheleaks-1.1.1+1.20.1` → `1.1.1`，而非 MC 版本号 `1.20.1`）
    if let Some(plus_pos) = name.find('+') {
        let prefix = &name[..plus_pos];
        for m in &matches {
            if prefix.contains(m.as_str()) {
                return m.clone();
            }
        }
    }
    // 否则取最后一个版本号（mod 版本号通常在 MC 版本号后面）
    // （如 `create-1.20.1-6.0.4` → `6.0.4`，而非 MC 版本号 `1.20.1`）
    matches.last().cloned().unwrap_or_default()
}

/// 从文件名或显示名提取 MC 版本号
///
/// 与 `extract_version_from_name` 不同：后者提取 mod 自身版本号（取最后一个标记），
/// 本函数只认"像 MC 版本"的标记（`1.x` 或新版本号 `2[6-9].x` 开头），取第一个，
/// 用于整合包 game_versions 为空时的兜底（如 `RLCraft 1.12.2 - Beta v2.8.1.zip` → `1.12.2`）。
pub fn extract_mc_version_from_name(name: &str) -> String {
    find_version_tokens(name)
        .into_iter()
        .find(|v| is_mc_version_like(v))
        .unwrap_or_default()
}

/// 形如 `1.12.2` / `1.20.1`，或 Minecraft 新版 `26.x` 开头（2[6-9]）
fn is_mc_version_like(ver: &str) -> bool {
    if ver.starts_with("1.") {
        return true;
    }
    let b = ver.as_bytes();
    b.len() >= 3
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == b'.'
        && (b[0] - b'0') * 10 + (b[1] - b'0') >= 26
}

#[cfg(test)]
#[path = "version_extract_test.rs"]
mod tests;
