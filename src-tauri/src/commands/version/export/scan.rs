//! 文件扫描 + 规则匹配
//! 目录扫描逻辑：
//! 1. 递归扫描实例目录（跳过 assets/versions/libraries 等大目录）
//! 2. 对每个文件，检查相对路径是否匹配用户勾选选项的规则
//! 3. 规则按顺序应用，`!` 开头表排除，后面的覆盖前面的

use std::path::Path;

use regex::Regex;

use super::options::GLOBAL_EXCLUDES;
use super::types::{ExportFileInfo, ExportOption};

/// 跳过的顶层目录（实例根目录下，又大又没用）
const SKIP_TOP_DIRS: &[&str] = &[
    "assets",
    "versions",
    "libraries",
    "structureCacheV1",
    ".fabric",
    ".git",
    "avatar-cache",
    "cosmetic-cache",
];

/// 收集所有需要导出的文件
///
/// 遍历 `instance_dir`，对每个文件检查是否匹配 `options` 中已勾选选项的规则。
/// 返回相对路径 + 绝对路径 + 文件大小。
pub fn collect_export_files(
    instance_dir: &Path,
    options: &[ExportOption],
) -> Result<Vec<ExportFileInfo>, String> {
    // 1. 合并所有已勾选选项的规则
    let rules = merge_checked_rules(options);
    if rules.is_empty() {
        return Ok(Vec::new());
    }

    log::debug!(
        "[Export] 合并后共 {} 条规则（含排除规则）",
        rules.len()
    );

    // 2. 编译规则为 regex
    let compiled = compile_rules(&rules)?;

    // 3. 递归扫描实例目录
    let mut files = Vec::new();
    scan_dir(
        instance_dir,
        instance_dir,
        &compiled,
        &mut files,
        true, // is_root
    )?;

    Ok(files)
}

/// 合并所有已勾选选项的规则（含全局排除规则）
fn merge_checked_rules(options: &[ExportOption]) -> Vec<String> {
    let mut rules: Vec<String> = Vec::new();

    // 收集已勾选选项的规则
    for opt in options {
        if !opt.checked || !opt.visible {
            continue;
        }
        if let Some(r) = &opt.rules {
            for part in r.split('|') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    rules.push(trimmed.to_string());
                }
            }
        }
    }

    // 追加全局排除规则
    for ex in GLOBAL_EXCLUDES {
        rules.push(ex.to_string());
    }

    rules
}

/// 单条编译后的规则（regex + 是否为排除规则）
struct CompiledRule {
    regex: Regex,
    is_exclude: bool,
}

/// 将通配符规则编译为 regex
///
/// 支持 `*`（任意字符）、`?`（单字符）、`[abc]`、`[!abc]`。
/// `!` 开头表排除。路径分隔符统一用 `/`。
fn compile_rules(rules: &[String]) -> Result<Vec<CompiledRule>, String> {
    let mut compiled = Vec::new();
    for rule in rules {
        let (is_exclude, pattern) = if let Some(stripped) = rule.strip_prefix('!') {
            (true, stripped)
        } else {
            (false, rule.as_str())
        };

        // 规则以 `/` 结尾表示匹配目录下所有内容（如 `mods/` 应匹配 `mods/xxx.jar`）
        // 直接传给 glob_to_regex 会生成 `^mods/$`，只匹配字面量 `mods/`，故补 `*`
        let pattern = if pattern.ends_with('/') {
            format!("{}*", pattern)
        } else {
            pattern.to_string()
        };

        let regex_str = glob_to_regex(&pattern);
        let regex = Regex::new(&regex_str)
            .map_err(|e| format!("编译规则失败 '{}': {}", pattern, e))?;
        compiled.push(CompiledRule { regex, is_exclude });
    }
    Ok(compiled)
}

/// 将通配符转为 regex 字符串
///
/// - `*` → `.*`
/// - `?` → `.`
/// - `[abc]` → `[abc]`
/// - `[!abc]` → `[^abc]`
/// - 其他 regex 特殊字符转义
fn glob_to_regex(glob: &str) -> String {
    let mut result = String::new();
    result.push('^');

    let chars: Vec<char> = glob.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' => {
                result.push_str(".*");
                i += 1;
            }
            '?' => {
                result.push('.');
                i += 1;
            }
            '[' => {
                // 字符类
                result.push('[');
                i += 1;
                if i < chars.len() && chars[i] == '!' {
                    result.push('^');
                    i += 1;
                }
                // 处理 `[]]` 这种边缘情况（`]` 作为第一个字符）
                while i < chars.len() && chars[i] != ']' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        result.push('\\');
                        result.push(chars[i + 1]);
                        i += 2;
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
                if i < chars.len() && chars[i] == ']' {
                    result.push(']');
                    i += 1;
                } else {
                    // 没有匹配的 `]`，视为字面量
                    result.push_str("\\[");
                }
            }
            c if "\\.+()|^${}".contains(c) => {
                result.push('\\');
                result.push(c);
                i += 1;
            }
            c => {
                result.push(c);
                i += 1;
            }
        }
    }

    result.push('$');
    result
}

/// 递归扫描目录，应用规则收集匹配的文件
fn scan_dir(
    root: &Path,
    current: &Path,
    rules: &[CompiledRule],
    files: &mut Vec<ExportFileInfo>,
    is_root: bool,
) -> Result<(), String> {
    let entries = std::fs::read_dir(current).map_err(|e| {
        format!("读取目录失败: {} ({})", current.display(), e)
    })?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // 根目录下跳过指定目录
            if is_root {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if SKIP_TOP_DIRS.contains(&name) {
                        continue;
                    }
                }
            }
            scan_dir(root, &path, rules, files, false)?;
        } else if path.is_file() {
            // 计算相对路径（正斜杠分隔）
            let rel = match path.strip_prefix(root) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let rel_str = rel.to_string_lossy().replace('\\', "/");

            // 检查规则：按顺序应用，后面的覆盖前面的
            let mut should_keep = false;
            for rule in rules {
                if rule.regex.is_match(&rel_str) {
                    should_keep = !rule.is_exclude;
                }
            }

            if should_keep {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                files.push(ExportFileInfo {
                    relative_path: rel_str,
                    abs_path: path,
                    size,
                });
            }
        }
    }

    Ok(())
}

/// 判断路径是否为 mod 文件（用于联网检查阶段筛选）
///
/// 压缩包/模组判定：扩展名为 .zip/.rar/.jar/.disabled/.old，且路径含
/// mods/packs/openloader/resource
pub fn is_mod_like_file(relative_path: &str) -> bool {
    let lower = relative_path.to_lowercase();
    let is_mod_ext = lower.ends_with(".zip")
        || lower.ends_with(".rar")
        || lower.ends_with(".jar")
        || lower.ends_with(".disabled")
        || lower.ends_with(".old");
    if !is_mod_ext {
        return false;
    }
    ["mods", "packs", "openloader", "resource"]
        .iter()
        .any(|s| lower.contains(s))
}