//! ECIES 加密信封协议
//!
//! 实现 MoSign-v1 业务接口的端到端加密信封：
//! ```json
//! { "payload": "<Base64Url AES-256-GCM 密文，前 12 字节为 nonce>",
//!   "key": "<Base64Url 临时 X25519 公钥 32 字节>" }
//! ```
//!
//! ## 加密流程（启动器 → 云端）
//!
//! 1. 启动器生成临时 X25519 密钥对 `(ephemeral_priv, ephemeral_pub)`
//! 2. ECDH：`shared = X25519(ephemeral_priv, device_public_key)`
//!    - `device_public_key` 是注册时云端返回的 X25519 公钥（云端为设备生成）
//! 3. HKDF-SHA256 派生 32 字节 AES 密钥（`salt = 空`，`info = "mosign-v1-ecies-envelope"`）
//! 4. AES-256-GCM 加密业务数据
//! 5. `payload = Base64Url(nonce(12B) || ciphertext)`
//! 6. `key = Base64Url(ephemeral_pub)`
//!
//! ## 解密流程（云端 → 启动器）
//!
//! 云端用相同协议加密响应，但 ECDH 改为 `X25519(server_ephemeral_priv, device.x25519_pub)`。
//! 启动器用本地 X25519 静态私钥 + 响应中的 `key`（临时公钥）解密。
//!
//! 协议参考：`api-server/docs/auth.md`「业务接口加密信封协议」章节

use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};

use super::crypto::{
    aes_gcm_decrypt, aes_gcm_encrypt, b64u_decode, b64u_encode, hkdf_sha256, x25519_public_from_b64u,
    CryptoError,
};

/// ECIES 信封 info 字符串（与服务端约定，不可修改）
const ECIES_INFO: &[u8] = b"mosign-v1-ecies-envelope";

/// ECIES 加密信封
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Base64Url AES-256-GCM 密文（前 12 字节为 nonce，后接 ciphertext + GCM tag）
    pub payload: String,
    /// Base64Url 临时 X25519 公钥（32 字节）
    pub key: String,
}

/// 加密结果（信封 + 临时私钥，私钥仅用于本会话，用完即弃）
pub struct SealedEnvelope {
    pub envelope: Envelope,
}

/// 用对方 X25519 公钥加密明文，生成 ECIES 信封
///
/// 参数：
/// - `plaintext`：业务数据明文
/// - `peer_public_b64u`：对方 X25519 公钥（Base64Url），即注册响应中的 `device_public_key`
pub fn seal(plaintext: &[u8], peer_public_b64u: &str) -> Result<SealedEnvelope, CryptoError> {
    let peer_public = x25519_public_from_b64u(peer_public_b64u)?;

    // 1. 生成临时 X25519 密钥对
    let mut csprng = OsRng;
    let ephemeral_secret = EphemeralSecret::random_from_rng(&mut csprng);
    let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

    // 2. ECDH 派生共享密钥
    let shared = ephemeral_secret.diffie_hellman(&peer_public).to_bytes();

    // 3. HKDF 派生 AES 密钥（salt = 空）
    let aes_key_bytes = hkdf_sha256(&shared, &[], ECIES_INFO, 32)?;
    let mut aes_key = [0u8; 32];
    aes_key.copy_from_slice(&aes_key_bytes);

    // 4. AES-256-GCM 加密
    let ciphertext = aes_gcm_encrypt(&aes_key, plaintext)?;

    Ok(SealedEnvelope {
        envelope: Envelope {
            payload: b64u_encode(&ciphertext),
            key: b64u_encode(&ephemeral_public.to_bytes()),
        },
    })
}

/// 用本地 X25519 静态私钥解密 ECIES 信封
///
/// 参数：
/// - `envelope`：收到的 ECIES 信封
/// - `our_secret_bytes`：本地持久化的 X25519 静态私钥（32 字节）
pub fn open(envelope: &Envelope, our_secret_bytes: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    // 1. 解析信封中的临时公钥
    let ephemeral_public_bytes = b64u_decode(&envelope.key)?;
    if ephemeral_public_bytes.len() != 32 {
        return Err(CryptoError::InvalidKeyLength {
            expected: 32,
            actual: ephemeral_public_bytes.len(),
        });
    }
    let mut pub_arr = [0u8; 32];
    pub_arr.copy_from_slice(&ephemeral_public_bytes);
    let ephemeral_public = X25519PublicKey::from(pub_arr);

    // 2. 用本地静态私钥做 ECDH
    let our_secret = x25519_dalek::StaticSecret::from(*our_secret_bytes);
    let shared = our_secret.diffie_hellman(&ephemeral_public).to_bytes();

    // 3. HKDF 派生 AES 密钥
    let aes_key_bytes = hkdf_sha256(&shared, &[], ECIES_INFO, 32)?;
    let mut aes_key = [0u8; 32];
    aes_key.copy_from_slice(&aes_key_bytes);

    // 4. AES-256-GCM 解密
    let ciphertext = b64u_decode(&envelope.payload)?;
    aes_gcm_decrypt(&aes_key, &ciphertext)
}

/// 判断 HTTP 响应体是否为 ECIES 加密信封
///
/// 加密响应 Content-Type 为 application/json，body 是 `{payload, key}` 结构。
/// 明文错误响应（401/400/500）不是信封，直接解析为 UnifiedResponse。
pub fn is_envelope(value: &serde_json::Value) -> bool {
    value
        .get("payload")
        .and_then(|v| v.as_str())
        .is_some()
        && value
            .get("key")
            .and_then(|v| v.as_str())
            .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minecraft::online::crypto::X25519StaticKeyPair;

    #[test]
    fn test_ecies_round_trip() {
        // 模拟云端：生成静态密钥对，公钥交给客户端
        let server_kp = X25519StaticKeyPair::generate();
        let server_public_b64u = server_kp.public_b64u();

        // 客户端加密（用云端公钥）
        let plaintext = br#"{"room_code":"AB3K7Q","max_players":4}"#;
        let sealed = seal(plaintext, &server_public_b64u).unwrap();

        // 云端解密（用自己私钥）
        let server_secret = server_kp.secret_bytes();
        let decrypted = open(&sealed.envelope, &server_secret).unwrap();
        assert_eq!(decrypted, plaintext);

        // 反向：云端加密（用客户端公钥），客户端解密
        let client_kp = X25519StaticKeyPair::generate();
        let client_public_b64u = client_kp.public_b64u();

        let sealed_resp = seal(b"response data", &client_public_b64u).unwrap();
        let client_secret = client_kp.secret_bytes();
        let decrypted_resp = open(&sealed_resp.envelope, &client_secret).unwrap();
        assert_eq!(decrypted_resp, b"response data");
    }

    #[test]
    fn test_is_envelope() {
        let envelope_json = serde_json::json!({
            "payload": "abc",
            "key": "def"
        });
        assert!(is_envelope(&envelope_json));

        let plain_json = serde_json::json!({
            "code": 1,
            "data": null,
            "msg": "ok"
        });
        assert!(!is_envelope(&plain_json));
    }
}
