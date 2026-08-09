//! 日志敏感信息脱敏
//!
//! 识别并替换日志中的 token、密钥等敏感信息，避免明文写入日志文件。

use regex::Regex;
use std::sync::OnceLock;

/// 对日志内容进行敏感信息脱敏
///
/// 识别并替换以下模式：
/// 1. JWT 格式 token：`eyJxxx.yyy.zzz`（三段，点分隔）
/// 2. JSON 中的敏感字段：`"token":"xxx"` / `"password":"xxx"` / `"api_key":"xxx"`
/// 3. `Authorization: Bearer <token>` 头
/// 4. URL query 中的敏感参数：`?token=xxx` / `?api_key=xxx`
///
/// 保留短字符串、URL、hash 等普通日志内容，只替换明确的敏感特征。
pub fn sanitize_sensitive_info(s: &str) -> String {
    static JWT_RE: OnceLock<Regex> = OnceLock::new();
    static JSON_TOKEN_RE: OnceLock<Regex> = OnceLock::new();
    static AUTH_HEADER_RE: OnceLock<Regex> = OnceLock::new();
    static URL_QUERY_RE: OnceLock<Regex> = OnceLock::new();

    let jwt_re = JWT_RE.get_or_init(|| {
        // JWT 格式：eyJ 开头，三段点分隔，每段至少 8 字符
        Regex::new(r"eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}").unwrap()
    });

    let json_token_re = JSON_TOKEN_RE.get_or_init(|| {
        // JSON 敏感字段：token / password / secret / api_key / authorization 等
        Regex::new(
            r#"(?i)"(access_token|accesstoken|refresh_token|refreshtoken|client_token|clienttoken|token|password|passwd|secret|api_key|apikey|client_secret|authorization|session)"\s*:\s*"[^"]{8,}""#,
        ).unwrap()
    });

    let auth_header_re = AUTH_HEADER_RE.get_or_init(|| {
        // Authorization: Bearer 头
        Regex::new(r"(?i)Authorization:\s*Bearer\s+[^\s,]+").unwrap()
    });

    let url_query_re = URL_QUERY_RE.get_or_init(|| {
        // URL query 敏感参数
        Regex::new(r#"([?&](token|key|api_key|apikey|signature|sig)=)[^&\s"'<>]+"#).unwrap()
    });

    let mut result = s.to_string();

    // 1. 替换 JWT 格式 token
    result = jwt_re.replace_all(&result, "***").to_string();

    // 2. 替换 JSON 中的敏感字段值
    result = json_token_re
        .replace_all(&result, r#""$1":"***""#)
        .to_string();

    // 3. 替换 Authorization: Bearer 头
    result = auth_header_re
        .replace_all(&result, "Authorization: Bearer ***")
        .to_string();

    // 4. 替换 URL query 敏感参数
    result = url_query_re.replace_all(&result, "$1***").to_string();

    result
}

#[cfg(test)]
#[path = "sanitize_tests.rs"]
mod tests;
