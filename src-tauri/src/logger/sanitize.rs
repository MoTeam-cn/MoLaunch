//! 日志敏感信息脱敏
//!
//! 识别并替换日志中的 token、密钥等敏感信息，避免明文写入日志文件。

use regex::Regex;
use std::sync::OnceLock;

/// 对日志内容进行敏感信息脱敏
///
/// 识别并替换以下模式：
/// 1. JWT 格式 token：`eyJxxx.yyy.zzz`（三段，点分隔）
/// 2. JSON 中的 token 字段：`"access_token":"xxx"` / `"accessToken":"xxx"`
///
/// 保留短字符串、URL、hash 等普通日志内容，只替换明确的 token 特征。
pub fn sanitize_sensitive_info(s: &str) -> String {
    static JWT_RE: OnceLock<Regex> = OnceLock::new();
    static JSON_TOKEN_RE: OnceLock<Regex> = OnceLock::new();

    let jwt_re = JWT_RE.get_or_init(|| {
        // JWT 格式：eyJ 开头，三段点分隔，每段至少 10 字符
        Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}").unwrap()
    });

    let json_token_re = JSON_TOKEN_RE.get_or_init(|| {
        // JSON 字段：access_token / accessToken / refresh_token / client_token / token
        Regex::new(
            r#"(?i)"(access_token|accesstoken|refresh_token|refreshtoken|client_token|clienttoken|session|token)"\s*:\s*"[^"]{8,}""#,
        ).unwrap()
    });

    let mut result = s.to_string();

    // 1. 替换 JWT 格式 token
    result = jwt_re.replace_all(&result, "***").to_string();

    // 2. 替换 JSON 中的 token 字段值
    result = json_token_re
        .replace_all(&result, r#""$1":"***""#)
        .to_string();

    result
}

#[cfg(test)]
#[path = "sanitize_tests.rs"]
mod tests;
