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
    fn test_sanitize_preserves_urls() {
        // URL 路径含 `/`，不应被误判为 token
        let input = "https://cdn-modrinth.mocdn.net/data/l9m9tuPN/versions/M8j2mfGj/physics-mod-3.0.14-mc-1.20.1-forge.jar";
        let result = sanitize_sensitive_info(input);
        assert_eq!(input, result, "URL should not be sanitized");
    }

    #[test]
    fn test_sanitize_preserves_texture_url_with_hex_hash() {
        // Minecraft 材质 URL 末尾是 64 字符 hex hash，不应被脱敏
        let input = "https://textures.minecraft.net/texture/a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let result = sanitize_sensitive_info(input);
        assert_eq!(input, result, "texture URL hash should not be sanitized");
    }
}
