//! 日志脱敏：将 token / 密码等敏感值替换为 ***
//!
//! 用于 frpc 日志输出前的脱敏处理，仅替换敏感值，保留日志结构和时间戳。

use once_cell::sync::Lazy;
use regex::Regex;

/// 敏感字段正则
///
/// 匹配模式（不区分大小写）：TOML `token = "xxx"` / JSON `"token":"xxx"` /
/// 通用 `token: xxx`。分组：1=字段名（保留），2=分隔符 `=` 或 `:`（保留），
/// 3=值（带引号或不带引号，替换为 `***`，保留引号风格）。
/// `\b` 确保只匹配完整字段名，不会误匹配 `my_token` 等带前缀的字段。
static SENSITIVE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)\b(token|password|secret|api_key|auth_token|access_token|refresh_token)\b["']?(\s*[:=]\s*)("[^"]*"|'[^']*'|[^\s,};#\]\n]+)"#,
    )
    .expect("敏感字段正则编译失败")
});

/// 对日志行进行脱敏
///
/// 将 `token`/`password`/`secret`/`api_key` 等敏感字段值替换为 `***`，保留字段名、
/// 分隔符和其余结构。支持 TOML/JSON/通用赋值三种格式。
///
/// ```
/// use mo_launch_lib::commands::frp::log_redact::redact_log;
/// assert_eq!(redact_log(r#"token = "secret""#), r#"token = "***""#);
/// assert_eq!(redact_log(r#"{"token":"abc","port":7000}"#), r#"{"token":"***","port":7000}"#);
/// ```
pub fn redact_log(line: &str) -> String {
    SENSITIVE_RE
        .replace_all(line, |caps: &regex::Captures| {
            let key = &caps[1];
            let sep = &caps[2];
            let val = &caps[3];
            // 保留引号风格：双引号值 → "***"，单引号值 → '***'，无引号值 → ***
            if val.starts_with('"') {
                format!("{}{}\"***\"", key, sep)
            } else if val.starts_with('\'') {
                format!("{}{}'***'", key, sep)
            } else {
                format!("{}{}***", key, sep)
            }
        })
        .into_owned()
}

#[cfg(test)]
#[path = "log_redact_tests.rs"]
mod tests;
