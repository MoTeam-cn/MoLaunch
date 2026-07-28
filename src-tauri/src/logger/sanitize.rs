//! 日志敏感信息脱敏
//!
//! 识别并替换日志中的 token、密钥等敏感信息，避免明文写入日志文件。

use regex::Regex;
use std::sync::OnceLock;

/// 对日志内容进行敏感信息脱敏
///
/// 识别并替换以下模式：
/// 1. JWT 格式 token：`eyJxxx.yyy.zzz`（三段，点分隔）
/// 2. Minecraft access_token：通常以 "eyJ" 开头的长字符串
/// 3. 长度 >= 40 的 hex/base64 字符串（可能是 token）
/// 4. JSON 中的 token 字段：`"access_token":"xxx"` / `"accessToken":"xxx"`
///
/// 保留短字符串和普通日志内容，只替换明显的 token 特征。
pub fn sanitize_sensitive_info(s: &str) -> String {
    static JWT_RE: OnceLock<Regex> = OnceLock::new();
    static JSON_TOKEN_RE: OnceLock<Regex> = OnceLock::new();
    static LONG_TOKEN_RE: OnceLock<Regex> = OnceLock::new();

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

    let long_token_re = LONG_TOKEN_RE.get_or_init(|| {
        // 长度 >= 40 的连续 base64/hex 字符串（可能是 token）
        // 字符集不含 `/`，避免把 URL 路径（如 net/data/xxx/versions/yyy/name）整体误判为 token
        // JWT 和 url-safe base64 token 不含 `/`，不受影响
        Regex::new(r"\b[A-Za-z0-9+=_-]{40,}\b").unwrap()
    });

    let mut result = s.to_string();

    // 1. 替换 JWT 格式 token
    result = jwt_re.replace_all(&result, "***").to_string();

    // 2. 替换 JSON 中的 token 字段值
    result = json_token_re
        .replace_all(&result, r#""$1":"***""#)
        .to_string();

    // 3. 替换超长 token 字符串（最后执行，避免误伤已脱敏的 ***）
    result = long_token_re.replace_all(&result, "***").to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_jwt() {
        let input = "Launching with token eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let result = sanitize_sensitive_info(input);
        assert!(!result.contains("eyJhbGciOiJIUzI1NiJ9"));
        assert!(result.contains("***"));
    }

    #[test]
    fn test_sanitize_json_token() {
        let input = r#"Auth response: {"access_token":"eyJsecret12345678","username":"player"}"#;
        let result = sanitize_sensitive_info(input);
        assert!(result.contains(r#""access_token":"***""#));
        assert!(!result.contains("eyJsecret12345678"));
        // username 不应被脱敏
        assert!(result.contains("player"));
    }

    #[test]
    fn test_sanitize_preserves_short_strings() {
        let input = "Game version: 1.16.5, Java path: C:/java/javaw.exe";
        let result = sanitize_sensitive_info(input);
        assert_eq!(input, result);
    }

    #[test]
    fn test_sanitize_long_token() {
        let input = "Token: abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
        let result = sanitize_sensitive_info(input);
        assert!(result.contains("***"));
        assert!(!result.contains("abc123def456ghi789"));
    }

    #[test]
    fn test_sanitize_preserves_urls() {
        // URL 路径含 `/`，不应被 long_token_re 误判为 token
        let input = "https://cdn-modrinth.mocdn.net/data/l9m9tuPN/versions/M8j2mfGj/physics-mod-3.0.14-mc-1.20.1-forge.jar";
        let result = sanitize_sensitive_info(input);
        assert_eq!(input, result, "URL should not be sanitized");
    }
}
