//! 版本号提取工具
//!
//! 从文件名或显示名中提取版本号，用于 CurseForge 等不直接提供版本号的平台。
//!
//! CurseForge 的 `Version` 字段为 `Nothing`，用 `Display`（即 `displayName`）
//! 作为 fallback 进行版本对比。
//!
//! CurseForge 的 `displayName` 通常类似：
//! - `jei-1.20.1-15.2.0.27.jar`
//! - `JustEnoughItems 1.20.1-15.2.0.27`
//! - `alltheleaks-1.1.1+1.20.1-forge.jar`
//!
//! 需要从中提取 mod 版本号（而非 MC 版本号）。

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
                } else if chars[end] == '.' && end + 1 < chars.len() && chars[end + 1].is_ascii_digit() {
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
