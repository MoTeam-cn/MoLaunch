//! ECIES 加密信封协议（MoSign-v1 业务接口端到端加密）
//!
//! 信封结构：`{payload: Base64Url(AES-256-GCM 密文, 前 12B 为 nonce), key: Base64Url(临时 X25519 公钥)}`。
//! 协议参考：`api-server/docs/auth.md`「业务接口加密信封协议」章节。

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
#[path = "ecies_tests.rs"]
mod tests;
