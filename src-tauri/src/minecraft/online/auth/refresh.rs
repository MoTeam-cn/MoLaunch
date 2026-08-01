//! 刷新流程：构造刷新请求 + 用 refresh 响应轮换设备凭证

use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::super::crypto::{
    aes_gcm_encrypt, b64u_decode, b64u_encode, hkdf_sha256, x25519_public_from_b64u, CryptoError,
    X25519StaticKeyPair,
};
use super::super::storage::DeviceCredentials;
use super::helpers::{generate_nonce_b64u, now_timestamp};
use super::types::{RefreshData, RefreshPayload, RefreshRequest};

/// 构造刷新请求（与 `build_login_request` 一致的 MoSign-v1 协议流程，payload 多 refresh_token 字段）
///
/// 参数：
/// - `creds`：已持久化的设备凭证（含 X25519 私钥、device_pk、device_public_key、refresh_token）
pub fn build_refresh_request(creds: &DeviceCredentials) -> Result<RefreshRequest, CryptoError> {
    // 1. 恢复本地 X25519 私钥
    let secret_bytes_vec = b64u_decode(&creds.x25519_secret_b64u)?;
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

    // 4. 生成 nonce + timestamp
    let timestamp = now_timestamp();
    let nonce = generate_nonce_b64u();
    let nonce_bytes = b64u_decode(&nonce)?;

    // 5. HKDF 派生 session_key
    let session_key_bytes = hkdf_sha256(&shared, &nonce_bytes, super::SESSION_KEY_INFO, 32)?;
    let mut session_key = [0u8; 32];
    session_key.copy_from_slice(&session_key_bytes);

    // 6. 构造 content 载荷（含 refresh_token，加密保护）
    let payload = RefreshPayload {
        device_pk: &creds.device_pk,
        timestamp,
        nonce: &nonce,
        refresh_token: &creds.refresh_token,
    };
    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| CryptoError::PemParseFailed(format!("序列化刷新载荷失败: {}", e)))?;

    crate::log_debug!(
        "[Online] 刷新 content 载荷长度={}B, device_pk={}, nonce={}",
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
    crate::log_debug!("[Online] 刷新请求构造完成");

    Ok(RefreshRequest {
        device_pk: creds.device_pk.clone(),
        v: super::PROTOCOL_VERSION,
        nonce,
        signature: signature_b64u,
        content: content_b64u,
        timestamp,
    })
}

/// 用 refresh 响应更新设备凭证（仅续期 access token + 轮换 refresh_token）
///
/// 服务端轮换 refresh_token：新 refresh_token 非空时替换本地存储，
/// 空（兼容老服务端未轮换）时保留原 refresh_token，过期时间统一续 30 天。
pub fn finalize_credentials_with_refresh(
    mut creds: DeviceCredentials,
    data: &RefreshData,
) -> DeviceCredentials {
    let now = now_timestamp();
    creds.device_token = data.device_token.clone();
    creds.token_expires_at = now + data.expires_in;
    if !data.refresh_token.is_empty() {
        creds.refresh_token = data.refresh_token.clone();
    }
    if !creds.refresh_token.is_empty() {
        creds.refresh_expires_at = now + super::REFRESH_TOKEN_TTL_SECS;
    }
    creds.last_login_at = now;
    creds
}
