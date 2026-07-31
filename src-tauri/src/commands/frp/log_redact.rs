//! 日志脱敏：将 token / 密码等敏感值替换为 ***
//!
//! 用于 frpc 日志输出前的脱敏处理，防止敏感信息写入日志文件或推送到前端。
//! 保留日志结构和时间戳，仅替换敏感值。
//!
//! 对应设计文档 §7 安全沙箱 - 阶段四「日志脱敏」。

use once_cell::sync::Lazy;
use regex::Regex;

/// 敏感字段正则
///
/// 匹配模式（不区分大小写）：
/// - TOML 风格：`token = "xxx"` / `token=xxx`
/// - JSON 风格：`"token":"xxx"` / `'token':'xxx'`
/// - 通用赋值：`token: xxx`
///
/// 分组：
/// 1. 字段名（保留原样，不替换）
/// 2. 分隔符 `=` 或 `:` 含两侧空白（保留原样）
/// 3. 值（带引号或不带引号，替换为 `***`，保留引号风格）
///
/// `\b` 确保只匹配完整字段名，不会误匹配 `my_token` 等带前缀的字段
/// （`_` 是单词字符，`\b` 在 `_` 与字母之间不产生边界）。
/// `["']?` 允许 JSON 风格中字段名后的闭合引号。
static SENSITIVE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)\b(token|password|secret|api_key|auth_token|access_token|refresh_token)\b["']?(\s*[:=]\s*)("[^"]*"|'[^']*'|[^\s,};#\]\n]+)"#,
    )
    .expect("敏感字段正则编译失败")
});

/// 对日志行进行脱敏
///
/// 将 `token` / `password` / `secret` / `api_key` 等敏感字段的值替换为 `***`，
/// 保留字段名、分隔符和日志其余结构（时间戳、级别等）。
///
/// 支持格式：
/// - TOML：`token = "value"` / `token=value`
/// - JSON：`"token":"value"` / `'token':'value'`
/// - 通用：`token: value`
///
/// # 示例
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
mod tests {
    use super::*;

    #[test]
    fn redacts_toml_quoted_token() {
        assert_eq!(redact_log(r#"token = "secret_value""#), r#"token = "***""#);
    }

    #[test]
    fn redacts_toml_unquoted_token() {
        assert_eq!(redact_log(r#"token=abc123"#), r#"token=***"#);
    }

    #[test]
    fn redacts_json_token() {
        let input = r#"{"token":"secret_value","port":7000}"#;
        let expected = r#"{"token":"***","port":7000}"#;
        assert_eq!(redact_log(input), expected);
    }

    #[test]
    fn redacts_single_quoted_value() {
        assert_eq!(redact_log(r#"token = 'secret'"#), r#"token = '***'"#);
    }

    #[test]
    fn redacts_password_field() {
        assert_eq!(redact_log(r#"password = "mypass""#), r#"password = "***""#);
    }

    #[test]
    fn redacts_secret_field() {
        assert_eq!(redact_log(r#"secret="topsecret""#), r#"secret="***""#);
    }

    #[test]
    fn redacts_api_key_field() {
        assert_eq!(redact_log(r#"api_key: "key123""#), r#"api_key: "***""#);
    }

    #[test]
    fn redacts_auth_token_field() {
        assert_eq!(
            redact_log(r#"auth_token = "abc""#),
            r#"auth_token = "***""#
        );
    }

    #[test]
    fn redacts_case_insensitive() {
        assert_eq!(redact_log(r#"TOKEN = "abc""#), r#"TOKEN = "***""#);
        assert_eq!(redact_log(r#"Password = "abc""#), r#"Password = "***""#);
    }

    #[test]
    fn preserves_log_structure() {
        let line = r#"[2026-07-31 12:00:00.123] [INFO] login with token = "abc" ok"#;
        let result = redact_log(line);
        assert!(result.starts_with("[2026-07-31 12:00:00.123] [INFO] login with token = "));
        assert!(result.contains("\"***\""));
        assert!(result.ends_with(" ok"));
        assert!(!result.contains("abc"));
    }

    #[test]
    fn does_not_redact_non_sensitive_fields() {
        let line = r#"server_addr = "example.com""#;
        assert_eq!(redact_log(line), line);
    }

    #[test]
    fn does_not_redact_prefixed_key() {
        // my_token 中 `_` 是单词字符，`\b` 不在 `_` 与 `t` 之间产生边界，
        // 因此不会误匹配 `my_token` 内部的 `token`。
        let line = r#"my_token = "value""#;
        assert_eq!(redact_log(line), line);
    }

    #[test]
    fn handles_multiple_sensitive_fields_in_one_line() {
        let line = r#"token = "a", password = "b""#;
        let result = redact_log(line);
        assert_eq!(result, r#"token = "***", password = "***""#);
    }

    #[test]
    fn empty_line_unchanged() {
        assert_eq!(redact_log(""), "");
    }

    #[test]
    fn line_without_sensitive_data_unchanged() {
        let line = "[INFO] frpc started on port 7000";
        assert_eq!(redact_log(line), line);
    }
}
