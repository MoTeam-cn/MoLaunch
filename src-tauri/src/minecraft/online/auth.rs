//! MoSign-v1 设备认证协议
//!
//! 实现 MoLaunch API Server 的设备注册/登录/登出流程。
//! 协议参考：`api-server/docs/auth.md`
//!
//! ## 注册流程（POST /v3/auth/register）
//!
//! 1. 启动器生成 Ed25519 + X25519 密钥对（持久化）
//! 2. 构造 content 载荷 JSON：`{ed25519_pub, x25519_pub, deviceid, timestamp, nonce}`
//! 3. 用云端 RSA 公钥 RSA-OAEP-SHA256 加密 content
//! 4. 签名材料 `"${payloadJson}.${nonce}.${timestamp}"`，用 Ed25519 私钥签名
//! 5. 提交 `{deviceid, v, noop=x25519_pub, nonce, signature, content, timestamp}`
//! 6. 持久化响应中的 `device_token`、`device_public_key`、`device_pk`
//!
//! ## 登录流程（POST /v3/auth/login）
//!
//! 1. 用本地 X25519 私钥 + 云端 X25519 公钥 ECDH 派生 shared
//! 2. HKDF-SHA256 派生 session_key（salt=nonce, info="mosign-v1-session-key"）
//! 3. 构造 content 载荷 JSON：`{device_pk, timestamp, nonce}`
//! 4. 用 session_key 做 AES-256-GCM 加密 content
//! 5. 用 session_key 做 HMAC-SHA256 签名 `"${payloadJson}.${nonce}.${timestamp}"`
//! 6. 提交 `{device_pk, v, nonce, signature, content, timestamp}`
//! 7. 持久化新 JWT

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use super::crypto::{
    b64u_decode, b64u_encode, hkdf_sha256, rsa_oaep_encrypt, CryptoError, Ed25519KeyPair,
    X25519StaticKeyPair,
};
use super::storage::DeviceCredentials;
use rand::RngCore;

/// 协议版本
pub const PROTOCOL_VERSION: &str = "MoSign-v1";

/// HKDF info for session key（与服务端约定）
const SESSION_KEY_INFO: &[u8] = b"mosign-v1-session-key";

/// 注册响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterResponse {
    pub code: u32,
    pub data: Option<RegisterData>,
    pub msg: String,
    pub time: String,
    pub req_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterData {
    /// JWT token
    pub device_token: String,
    /// 云端为设备生成的 X25519 公钥（Base64Url，ECIES 加密用）
    pub device_public_key: String,
    /// 设备主键（UUID）
    pub device_pk: String,
    /// JWT 有效期（秒）
    pub expires_in: u64,
}

/// 登录响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginResponse {
    pub code: u32,
    pub data: Option<LoginData>,
    pub msg: String,
    pub time: String,
    pub req_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginData {
    pub device_token: String,
    pub expires_in: u64,
}

/// 注册载荷（content 解密后的明文 JSON）
#[derive(Debug, Serialize)]
struct RegisterPayload<'a> {
    ed25519_pub: &'a str,
    x25519_pub: &'a str,
    deviceid: &'a str,
    timestamp: u64,
    nonce: &'a str,
}

/// 登录载荷（content 解密后的明文 JSON）
#[derive(Debug, Serialize)]
struct LoginPayload<'a> {
    device_pk: &'a str,
    timestamp: u64,
    nonce: &'a str,
}

/// 注册请求体
#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub deviceid: String,
    pub v: &'static str,
    pub noop: String,
    pub nonce: String,
    pub signature: String,
    pub content: String,
    pub timestamp: u64,
}

/// 登录请求体
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub device_pk: String,
    pub v: &'static str,
    pub nonce: String,
    pub signature: String,
    pub content: String,
    pub timestamp: u64,
}

/// 生成 16 字节随机 nonce（Base64Url）
fn generate_nonce_b64u() -> String {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    b64u_encode(&bytes)
}

/// 当前 Unix 时间戳（秒）
fn now_timestamp() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

/// 生成设备友好标识 `mcsdk-xxxx-xxxx-xxxx-xxxx`（小写十六进制）
pub fn generate_device_id() -> String {
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let hex = hex::encode(bytes);
    format!(
        "mcsdk-{}-{}-{}-{}",
        &hex[0..4],
        &hex[4..8],
        &hex[8..12],
        &hex[12..16]
    )
}

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

    crate::log_debug!(
        "[Online] 注册 content 载荷长度={}B, deviceid={}, nonce={}",
        payload_json.len(),
        device_id,
        nonce
    );

    // RSA-OAEP 加密 content
    let encrypted_content = rsa_oaep_encrypt(server_rsa_pem, payload_json.as_bytes())?;
    let content_b64u = b64u_encode(&encrypted_content);

    // 签名材料
    let sign_material = format!("{}.{}.{}", payload_json, nonce, timestamp);
    let signature_b64u = kp.ed25519.sign_b64u(sign_material.as_bytes());
    crate::log_debug!("[Online] 注册请求构造完成，签名材料长度={}B", sign_material.len());

    let req = RegisterRequest {
        deviceid: device_id.to_string(),
        v: PROTOCOL_VERSION,
        noop: x25519_pub_b64u.clone(),
        nonce: nonce.clone(),
        signature: signature_b64u,
        content: content_b64u,
        timestamp,
    };

    // 凭证预填充（device_token / device_pk / device_public_key 由响应填入）
    let mut creds = DeviceCredentials::default();
    creds.ed25519_seed_b64u = b64u_encode(&kp.ed25519.seed());
    creds.x25519_secret_b64u = b64u_encode(&kp.x25519.secret_bytes());
    creds.device_id = device_id.to_string();

    Ok((req, creds))
}

/// 用注册响应完善设备凭证
pub fn finalize_credentials_with_register(
    mut creds: DeviceCredentials,
    resp: &RegisterData,
) -> DeviceCredentials {
    creds.device_pk = resp.device_pk.clone();
    creds.device_token = resp.device_token.clone();
    creds.device_public_key_b64u = resp.device_public_key.clone();
    creds.token_expires_at = now_timestamp() + resp.expires_in;
    creds.last_login_at = now_timestamp();
    creds
}

/// 构造登录请求
///
/// 参数：
/// - `creds`：已持久化的设备凭证（含 X25519 私钥、device_pk、device_public_key）
pub fn build_login_request(
    creds: &DeviceCredentials,
) -> Result<LoginRequest, CryptoError> {
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
    let peer_public = super::crypto::x25519_public_from_b64u(&creds.device_public_key_b64u)?;

    // 3. ECDH 派生共享密钥
    let shared = our_x25519.diffie_hellman(&peer_public);

    // 4. 生成 nonce
    let timestamp = now_timestamp();
    let nonce = generate_nonce_b64u();
    let nonce_bytes = b64u_decode(&nonce)?;

    // 5. HKDF 派生 session_key
    let session_key_bytes = hkdf_sha256(&shared, &nonce_bytes, SESSION_KEY_INFO, 32)?;
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
    let encrypted_content = super::crypto::aes_gcm_encrypt(&session_key, payload_json.as_bytes())?;
    let content_b64u = b64u_encode(&encrypted_content);

    // 8. HMAC-SHA256 签名
    let sign_material = format!("{}.{}.{}", payload_json, nonce, timestamp);
    let mut hmac = Hmac::<Sha256>::new_from_slice(&session_key)
        .map_err(|_| CryptoError::HkdfExpandFailed)?;
    hmac.update(sign_material.as_bytes());
    let signature_bytes = hmac.finalize().into_bytes();
    let signature_b64u = b64u_encode(&signature_bytes);
    crate::log_debug!("[Online] 登录请求构造完成");

    Ok(LoginRequest {
        device_pk: creds.device_pk.clone(),
        v: PROTOCOL_VERSION,
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
    creds.device_token = resp.device_token.clone();
    creds.token_expires_at = now_timestamp() + resp.expires_in;
    creds.last_login_at = now_timestamp();
    creds
}

/// 复用密钥对容器（注册时一次性生成 Ed25519 + X25519）
pub struct OnlineKeyPair {
    pub ed25519: Ed25519KeyPair,
    pub x25519: X25519StaticKeyPair,
}

impl OnlineKeyPair {
    /// 生成新的密钥对组合
    pub fn generate() -> Self {
        Self {
            ed25519: Ed25519KeyPair::generate(),
            x25519: X25519StaticKeyPair::generate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_device_id_format() {
        let id = generate_device_id();
        assert!(id.starts_with("mcsdk-"));
        // 4 段，每段 4 字符十六进制
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], "mcsdk");
        for part in &parts[1..] {
            assert_eq!(part.len(), 4);
            assert!(part.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn test_build_login_request_round_trip() {
        // 模拟设备已注册的状态
        let kp = OnlineKeyPair::generate();
        let mut creds = DeviceCredentials::default();
        creds.ed25519_seed_b64u = b64u_encode(&kp.ed25519.seed());
        creds.x25519_secret_b64u = b64u_encode(&kp.x25519.secret_bytes());
        creds.device_pk = "test-device-pk".to_string();
        creds.device_public_key_b64u = kp.x25519.public_b64u(); // 用自己公钥模拟云端公钥（仅测试流程）
        creds.device_id = "mcsdk-test".to_string();

        // 由于云端公钥 = 自己公钥，ECDH 会产生 shared（虽然不真实，但流程可走通）
        let req = build_login_request(&creds);
        // 由于 ECDH 需要真正的对方公钥，这里可能失败，但能验证流程
        assert!(req.is_ok(), "登录请求构造应成功");
        let req = req.unwrap();
        assert_eq!(req.device_pk, "test-device-pk");
        assert_eq!(req.v, PROTOCOL_VERSION);
        assert!(!req.signature.is_empty());
        assert!(!req.content.is_empty());
    }
}
