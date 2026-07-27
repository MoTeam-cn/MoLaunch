//! 加密原语模块
//!
//! 提供 MoSign-v1 协议所需的全部加密原语：
//! - Base64Url 编码/解码（无填充）
//! - Ed25519 密钥对生成、签名
//! - X25519 密钥对生成、ECDH 共享密钥派生
//! - HKDF-SHA256 密钥派生
//! - AES-256-GCM 加密/解密
//! - RSA-OAEP-SHA256 加密（用云端 RSA 公钥加密注册 content）
//!
//! 算法参考：`api-server/docs/auth.md`「算法清单」

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use rsa::{Oaep, RsaPublicKey};
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

/// Base64Url 编码（无填充）
pub fn b64u_encode(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

/// Base64Url 解码（无填充）
pub fn b64u_decode(s: &str) -> Result<Vec<u8>, CryptoError> {
    URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|e| CryptoError::Base64Decode(e))
}

// ============================== Ed25519 ==============================

/// Ed25519 密钥对（私钥 + 公钥）
pub struct Ed25519KeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Ed25519KeyPair {
    /// 生成新的 Ed25519 密钥对
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// 从 32 字节私钥种子恢复
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// 私钥种子（32 字节）
    pub fn seed(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// 公钥（32 字节）
    pub fn public_key(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// 公钥 Base64Url
    pub fn public_key_b64u(&self) -> String {
        b64u_encode(&self.public_key())
    }

    /// 签名（64 字节）
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let sig: Signature = self.signing_key.sign(message);
        sig.to_bytes()
    }

    /// 签名 Base64Url
    pub fn sign_b64u(&self, message: &[u8]) -> String {
        b64u_encode(&self.sign(message))
    }
}

// ============================== X25519 ==============================

/// X25519 静态密钥对（用于持久化的设备 X25519 私钥）
pub struct X25519StaticKeyPair {
    pub secret: StaticSecret,
    pub public: X25519PublicKey,
}

impl X25519StaticKeyPair {
    /// 生成新的 X25519 静态密钥对
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let secret = StaticSecret::random_from_rng(&mut csprng);
        let public = X25519PublicKey::from(&secret);
        Self { secret, public }
    }

    /// 从 32 字节私钥恢复
    pub fn from_bytes(secret_bytes: &[u8; 32]) -> Self {
        let secret = StaticSecret::from(*secret_bytes);
        let public = X25519PublicKey::from(&secret);
        Self { secret, public }
    }

    /// 私钥（32 字节）
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.secret.to_bytes()
    }

    /// 公钥（32 字节）
    pub fn public_bytes(&self) -> [u8; 32] {
        self.public.to_bytes()
    }

    /// 公钥 Base64Url
    pub fn public_b64u(&self) -> String {
        b64u_encode(&self.public_bytes())
    }

    /// 与对方公钥做 ECDH，派生 32 字节共享密钥
    pub fn diffie_hellman(&self, peer_public: &X25519PublicKey) -> [u8; 32] {
        self.secret.diffie_hellman(peer_public).to_bytes()
    }
}

/// 从 Base64Url 公钥恢复 X25519 PublicKey
pub fn x25519_public_from_b64u(s: &str) -> Result<X25519PublicKey, CryptoError> {
    let bytes = b64u_decode(s)?;
    if bytes.len() != 32 {
        return Err(CryptoError::InvalidKeyLength {
            expected: 32,
            actual: bytes.len(),
        });
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(X25519PublicKey::from(arr))
}

// ============================== HKDF ==============================

/// HKDF-SHA256 密钥派生
///
/// 参数：
/// - `ikm`：输入密钥材料（如 ECDH 共享密钥）
/// - `salt`：盐值（如 nonce 原始字节）
/// - `info`：上下文信息（如 "mosign-v1-session-key"）
/// - `length`：输出长度（默认 32）
pub fn hkdf_sha256(ikm: &[u8], salt: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>, CryptoError> {
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut okm = vec![0u8; length];
    hk.expand(info, &mut okm)
        .map_err(|_| CryptoError::HkdfExpandFailed)?;
    Ok(okm)
}

// ============================== AES-256-GCM ==============================

/// AES-256-GCM 加密
///
/// 返回 `nonce(12B) || ciphertext`（GCM tag 嵌入 ciphertext 末尾）
pub fn aes_gcm_encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::AesGcmEncryptFailed)?;
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// AES-256-GCM 解密
///
/// 输入 `nonce(12B) || ciphertext`，返回明文
pub fn aes_gcm_decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 12 {
        return Err(CryptoError::InvalidCiphertextLength);
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::AesGcmDecryptFailed)
}

// ============================== RSA-OAEP ==============================

/// RSA-OAEP-SHA256 加密
///
/// 用于注册时用云端 RSA 公钥加密 content。
/// 公钥为 PEM SPKI 格式。
///
/// 长度限制：RSA-OAEP-SHA256 最大明文 = (模数字节数) - 2*32 - 2
/// - RSA-2048 (256B 模数): 最大 190 字节
/// - RSA-3072 (384B 模数): 最大 318 字节
/// - RSA-4096 (512B 模数): 最大 446 字节
///
/// 注册 content JSON 约 209 字节，**RSA-2048 不足以承载**，
/// api-server 必须使用 RSA-3072 或更高位数。
pub fn rsa_oaep_encrypt(public_pem: &str, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    use rsa::pkcs8::DecodePublicKey;
    use rsa::traits::PublicKeyParts;
    let public_key = RsaPublicKey::from_public_key_pem(public_pem)
        .map_err(|e| CryptoError::RsaKeyParseFailed(e.to_string()))?;

    let key_bits = public_key.n().bits();
    let max_plain = (key_bits / 8) - 2 * 32 - 2;
    crate::log_debug!(
        "[Online] RSA-OAEP 加密: 公钥位数={}bit, 明文长度={}B, 最大允许={}B",
        key_bits,
        plaintext.len(),
        max_plain
    );

    if plaintext.len() > max_plain {
        let msg = format!(
            "明文长度 {} 字节超过 RSA-{} + OAEP-SHA256 最大允许 {} 字节；\
             请在 api-server 端重新生成 RSA-3072 或 RSA-4096 密钥（当前为 RSA-{}）",
            plaintext.len(),
            key_bits,
            max_plain,
            key_bits
        );
        crate::log_error!("[Online] RSA 加密失败: {}", msg);
        return Err(CryptoError::RsaEncryptFailed(msg));
    }

    let mut rng = OsRng;
    let padding = Oaep::new::<Sha256>();
    public_key
        .encrypt(&mut rng, padding, plaintext)
        .map_err(|e| {
            crate::log_error!("[Online] RSA 加密底层失败: {}", e);
            CryptoError::RsaEncryptFailed(e.to_string())
        })
}

// ============================== 错误类型 ==============================

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Base64Url 解码失败: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("密钥长度错误: 期望 {expected}, 实际 {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("HKDF 展开失败")]
    HkdfExpandFailed,

    #[error("AES-GCM 加密失败")]
    AesGcmEncryptFailed,

    #[error("AES-GCM 解密失败")]
    AesGcmDecryptFailed,

    #[error("密文长度不足（小于 12 字节 nonce）")]
    InvalidCiphertextLength,

    #[error("PEM 解析失败: {0}")]
    PemParseFailed(String),

    #[error("RSA 公钥解析失败: {0}")]
    RsaKeyParseFailed(String),

    #[error("RSA 加密失败: {0}")]
    RsaEncryptFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;

    #[test]
    fn test_ed25519_sign_verify() {
        let kp = Ed25519KeyPair::generate();
        let msg = b"hello world";
        let sig = kp.sign(msg);
        // 验证签名
        let verifying_key = kp.verifying_key;
        verifying_key
            .verify(msg, &Signature::from_bytes(&sig))
            .expect("signature should verify");
    }

    #[test]
    fn test_x25519_ecdh_round_trip() {
        let alice = X25519StaticKeyPair::generate();
        let bob = X25519StaticKeyPair::generate();
        let shared_a = alice.diffie_hellman(&bob.public);
        let shared_b = bob.diffie_hellman(&alice.public);
        assert_eq!(shared_a, shared_b, "ECDH 共享密钥应一致");
    }

    #[test]
    fn test_aes_gcm_round_trip() {
        let key = [42u8; 32];
        let plaintext = b"secret payload";
        let ciphertext = aes_gcm_encrypt(&key, plaintext).unwrap();
        let decrypted = aes_gcm_decrypt(&key, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_hkdf_deterministic() {
        let ikm = [1u8; 32];
        let salt = [2u8; 16];
        let info = b"mosign-v1-session-key";
        let k1 = hkdf_sha256(&ikm, &salt, info, 32).unwrap();
        let k2 = hkdf_sha256(&ikm, &salt, info, 32).unwrap();
        assert_eq!(k1, k2, "相同输入应派生相同密钥");
    }
}
