//! PKCE（RFC 7636）工具：code_verifier 生成 + S256 code_challenge
//!
//! 桌面应用作为公开客户端（public client）不持有 client_secret 时，
//! 使用 PKCE 在本地生成一次性随机 verifier，授权请求附带其 S256 摘要，
//! token 交换时回传原始 verifier，由授权服务器校验，防中间人窃取授权码。

use rand::Rng;

/// 生成 PKCE code_verifier
///
/// RFC 7636 要求 43-128 位字符，且仅含 `[A-Za-z0-9._~-]`。
/// 采用 96 位随机字符，在范围内留足熵。
pub fn generate_code_verifier() -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    let len = 96usize;
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect()
}

/// 计算 S256 code_challenge
///
/// `BASE64URL-ENCODE(SHA256(ASCII(code_verifier)))`，不带 padding。
pub fn code_challenge_s256(code_verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let digest = hasher.finalize();
    base64_url_nopad(&digest)
}

/// Base64 URL-safe 无填充编码（RFC 4648 §5，用于 PKCE challenge）
fn base64_url_nopad(bytes: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_length_and_charset() {
        let verifier = generate_code_verifier();
        assert!(verifier.len() >= 43 && verifier.len() <= 128);
        assert!(verifier.chars().all(|c| c.is_ascii_alphanumeric()
            || c == '-'
            || c == '.'
            || c == '_'
            || c == '~'));
    }

    #[test]
    fn test_verifier_unique() {
        let a = generate_code_verifier();
        let b = generate_code_verifier();
        assert_ne!(a, b);
    }

    #[test]
    fn test_challenge_deterministic() {
        // RFC 7636 附录 B 示例
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert_eq!(code_challenge_s256(verifier), expected);
    }

    #[test]
    fn test_challenge_nopad_urlsafe() {
        let verifier = generate_code_verifier();
        let challenge = code_challenge_s256(&verifier);
        assert!(!challenge.contains('='));
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
    }
}
