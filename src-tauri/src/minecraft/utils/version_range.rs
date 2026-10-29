//! 版本声明匹配
//!
//! 模组与平台给出的兼容声明不是干净的「一个版本号」：
//! 可能是 `1.20`、`1.20.x`、`>=1.20 <1.21`，也可能是一串空格分隔的条件。
//! 原先的过滤只做字符串相等，于是 `1.20.x` 这种写法永远匹配不上。
//!
//! 保守原则：看不懂的声明**不算匹配**。宁可漏装一个模组，也不要装上一个
//! 声明了不兼容的模组——后者会在启动时炸，而且原因离安装环节很远。

/// 比较两个版本号：返回 Ordering 语义的 -1 / 0 / 1
///
/// 只比数字段，非数字后缀（`1.20.1-pre1` 的 `pre1`）忽略：
/// 这里的用途是「能不能装」，不是精确排序。
fn compare(a: &str, b: &str) -> i32 {
    let left = numbers(a);
    let right = numbers(b);
    for index in 0..left.len().max(right.len()) {
        let l = *left.get(index).unwrap_or(&0);
        let r = *right.get(index).unwrap_or(&0);
        if l != r {
            return if l < r { -1 } else { 1 };
        }
    }
    0
}

fn numbers(version: &str) -> Vec<u64> {
    version
        .split(['.', '-', '+'])
        .map_while(|part| part.parse::<u64>().ok())
        .collect()
}

/// 单条声明是否覆盖目标版本
///
/// 支持：精确（`1.20.1`）、段通配（`1.20.x`）、比较符（`>=1.20 <1.21`）、
/// 任意（`*`）。空格分隔视为「同时满足」。
pub(crate) fn matches_game(declared: &str, target: &str) -> bool {
    let declared = declared.trim();
    if declared.is_empty() {
        return false;
    }
    if declared == "*" || declared.eq_ignore_ascii_case("any") {
        return true;
    }
    declared
        .split_whitespace()
        .all(|token| matches_token(token, target))
}

fn matches_token(token: &str, target: &str) -> bool {
    let token = token.trim();
    if token.is_empty() {
        return true;
    }
    if let Some(rest) = token.strip_prefix(">=") {
        return compare(target, rest) >= 0;
    }
    if let Some(rest) = token.strip_prefix("<=") {
        return compare(target, rest) <= 0;
    }
    if let Some(rest) = token.strip_prefix('>') {
        return compare(target, rest) > 0;
    }
    if let Some(rest) = token.strip_prefix('<') {
        return compare(target, rest) < 0;
    }
    if let Some(rest) = token.strip_prefix('=') {
        return compare(target, rest) == 0;
    }
    if token.contains('x') || token.contains('X') || token.contains('*') {
        return matches_wildcard(token, target);
    }
    compare(token, target) == 0
}

/// `1.20.x` 这类段通配：只有声明里的数字段参与比较
fn matches_wildcard(declared: &str, target: &str) -> bool {
    let declared_numbers = numbers(declared);
    if declared_numbers.is_empty() {
        return false;
    }
    let target_numbers = numbers(target);
    if target_numbers.len() < declared_numbers.len() {
        return false;
    }
    declared_numbers
        .iter()
        .zip(target_numbers.iter())
        .all(|(declared, target)| declared == target)
}

#[cfg(test)]
#[path = "version_range_test.rs"]
mod version_range_test;
