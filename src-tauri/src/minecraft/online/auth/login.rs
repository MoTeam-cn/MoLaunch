//! 登录流程：构造登录请求 + 用登录响应更新设备凭证

use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::super::crypto::{
    aes_gcm_encrypt, b64u_decode, b64u_encode, hkdf_sha256, x25519_public_from_b64u, CryptoError,
    X25519StaticKeyPair,
};
use super::super::storage::DeviceCredentials;
use super::helpers::{generate_nonce_b64u, now_timestamp};
use super::types::{LoginData, LoginPayload, LoginRequest};

/// 构造登录请求
///
/// 参数：
/// - `creds`：已持久化的设备凭证（含 X25519 私钥、device_pk、device_public_key）
pub fn build_login_request(creds: &DeviceCredentials) -> Result<LoginRequest, CryptoError> {
    // 1. 恢复本地 X25519 私钥
    let secret_bytes_b64u = &creds.x25519_secret_b64u;
    let secret_bytes_vec = b64u_decode(secret_bytes_b64u)?;
    if secret_bytes_vec.len() != 32 {
        return Err(CryptoError::InvalidKeyLength {
            expected: 32,
            actual: secret_bytes_vec.len(),
        });
    }
    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&secret_bytes_vec);
    let our_x25519 = X25519StaticKeyPair::from_bytes(&secret_bytes);

    // 2. 解析云端 X25519 公钥
    let peer_public = x25519_public_from_b64u(&creds.device_public_key_b64u)?;

    // 3. ECDH 派生共享密钥
    let shared = our_x25519.diffie_hellman(&peer_public);

    // 4. 生成 nonce
    let timestamp = now_timestamp();
    let nonce = generate_nonce_b64u();
    let nonce_bytes = b64u_decode(&nonce)?;

    // 5. HKDF 派生 session_key
    let session_key_bytes = hkdf_sha256(&shared, &nonce_bytes, super::SESSION_KEY_INFO, 32)?;
    let mut session_key = [0u8; 32];
    session_key.copy_from_slice(&session_key_bytes);

    // 6. 构造 content 载荷
    let payload = LoginPayload {
        device_pk: &creds.device_pk,
        timestamp,
        nonce: &nonce,
    };
    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| CryptoError::PemParseFailed(format!("序列化登录载荷失败: {}", e)))?;

    crate::log_debug!(
        "[Online] 登录 content 载荷长度={}B, device_pk={}, nonce={}",
        payload_json.len(),
        creds.device_pk,
        nonce
    );

    // 7. AES-256-GCM 加密 content
    let encrypted_content = aes_gcm_encrypt(&session_key, payload_json.as_bytes())?;
    let content_b64u = b64u_encode(&encrypted_content);

    // 8. HMAC-SHA256 签名
    let sign_material = format!("{}.{}.{}", payload_json, nonce, timestamp);
    let mut hmac =
        Hmac::<Sha256>::new_from_slice(&session_key).map_err(|_| CryptoError::HkdfExpandFailed)?;
    hmac.update(sign_material.as_bytes());
    let signature_bytes = hmac.finalize().into_bytes();
    let signature_b64u = b64u_encode(&signature_bytes);
    crate::log_debug!("[Online] 登录请求构造完成");

    Ok(LoginRequest {
        device_pk: creds.device_pk.clone(),
        v: super::PROTOCOL_VERSION,
        nonce,
        signature: signature_b64u,
        content: content_b64u,
        timestamp,
    })
}

/// 用登录响应更新设备凭证（仅刷新 token）
pub fn finalize_credentials_with_login(
    mut creds: DeviceCredentials,
    resp: &LoginData,
) -> DeviceCredentials {
    let now = now_timestamp();
    creds.device_token = resp.device_token.clone();
    creds.token_expires_at = now + resp.expires_in;
    creds.refresh_token = resp.refresh_token.clone();
    creds.refresh_expires_at = if resp.refresh_token.is_empty() {
        0
    } else {
        now + super::REFRESH_TOKEN_TTL_SECS
    };
    creds.last_login_at = now;
    creds
}
