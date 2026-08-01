//! 注册流程：构造注册请求 + 用注册响应完善设备凭证

use super::super::crypto::{b64u_encode, rsa_oaep_encrypt, CryptoError};
use super::super::storage::DeviceCredentials;
use super::helpers::{generate_nonce_b64u, now_timestamp};
use super::keypair::OnlineKeyPair;
use super::types::{RegisterData, RegisterPayload, RegisterRequest};

/// 构造注册请求
///
/// 参数：
/// - `kp`：本地生成的 Ed25519 + X25519 密钥对
/// - `device_id`：设备友好标识
/// - `server_rsa_pem`：云端 RSA 公钥 PEM（SPKI 格式）
///
/// 返回：(RegisterRequest, 派生的 DeviceCredentials 用于持久化)
pub fn build_register_request(
    kp: &OnlineKeyPair,
    device_id: &str,
    server_rsa_pem: &str,
) -> Result<(RegisterRequest, DeviceCredentials), CryptoError> {
    let timestamp = now_timestamp();
    let nonce = generate_nonce_b64u();

    // content 载荷
    let ed25519_pub_b64u = kp.ed25519.public_key_b64u();
    let x25519_pub_b64u = kp.x25519.public_b64u();
    let payload = RegisterPayload {
        ed25519_pub: &ed25519_pub_b64u,
        x25519_pub: &x25519_pub_b64u,
        deviceid: device_id,
        timestamp,
        nonce: &nonce,
    };
    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| CryptoError::PemParseFailed(format!("序列化注册载荷失败: {}", e)))?;

    crate::log_info!(
        "[Online] 注册 content 载荷: 长度={}B, deviceid={}, nonce={}, timestamp={}",
        payload_json.len(),
        device_id,
        nonce,
        timestamp
    );

    // RSA-OAEP 加密 content
    let encrypted_content = rsa_oaep_encrypt(server_rsa_pem, payload_json.as_bytes())?;
    let content_b64u = b64u_encode(&encrypted_content);

    // 签名材料
    let sign_material = format!("{}.{}.{}", payload_json, nonce, timestamp);
    let signature_b64u = kp.ed25519.sign_b64u(sign_material.as_bytes());
    crate::log_info!(
        "[Online] 注册请求构造完成: 签名材料长度={}B, content密文长度={}B",
        sign_material.len(),
        encrypted_content.len()
    );

    let req = RegisterRequest {
        deviceid: device_id.to_string(),
        v: super::PROTOCOL_VERSION,
        noop: x25519_pub_b64u.clone(),
        nonce: nonce.clone(),
        signature: signature_b64u,
        content: content_b64u,
        timestamp,
    };

    // 凭证预填充（device_token / device_pk / device_public_key 由响应填入）
    let creds = DeviceCredentials {
        ed25519_seed_b64u: b64u_encode(&kp.ed25519.seed()),
        x25519_secret_b64u: b64u_encode(&kp.x25519.secret_bytes()),
        device_id: device_id.to_string(),
        ..Default::default()
    };

    Ok((req, creds))
}

/// 用注册响应完善设备凭证
pub fn finalize_credentials_with_register(
    mut creds: DeviceCredentials,
    resp: &RegisterData,
) -> DeviceCredentials {
    let now = now_timestamp();
    creds.device_pk = resp.device_pk.clone();
    creds.device_token = resp.device_token.clone();
    creds.device_public_key_b64u = resp.device_public_key.clone();
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
