//! MoSign-v1 协议类型定义（注册/登录/刷新的请求与响应）

use serde::{Deserialize, Serialize};

/// 注册响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterResponse {
    pub code: u32,
    pub data: Option<RegisterData>,
    pub msg: String,
    #[serde(default)]
    pub time: String,
    #[serde(default)]
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
    /// refresh_token（用于续期 access token，30 天有效期）
    #[serde(default)]
    pub refresh_token: String,
}

/// 登录响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginResponse {
    pub code: u32,
    pub data: Option<LoginData>,
    pub msg: String,
    #[serde(default)]
    pub time: String,
    #[serde(default)]
    pub req_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginData {
    pub device_token: String,
    pub expires_in: u64,
    /// refresh_token（用于续期 access token，30 天有效期）
    #[serde(default)]
    pub refresh_token: String,
}

/// refresh 请求体（与 `LoginRequest` 完全一致的 MoSign-v1 协议结构）
///
/// `refresh_token` 放在加密的 content 内（即 `RefreshPayload`），明文不出现在请求体。
/// 即使 refresh_token 泄露，攻击者无设备 X25519 私钥也无法构造合法请求。
#[derive(Debug, Clone, Serialize)]
pub struct RefreshRequest {
    pub device_pk: String,
    pub v: &'static str,
    pub nonce: String,
    pub signature: String,
    pub content: String,
    pub timestamp: u64,
}

/// refresh 响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefreshResponse {
    pub code: u32,
    pub data: Option<RefreshData>,
    pub msg: String,
    #[serde(default)]
    pub time: String,
    #[serde(default)]
    pub req_id: String,
}

/// refresh 响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefreshData {
    /// 新的 access token
    pub device_token: String,
    /// 新的 refresh_token（服务端轮换）
    #[serde(default)]
    pub refresh_token: String,
    /// access token 有效期（秒）
    pub expires_in: u64,
}

/// 注册载荷（content 解密后的明文 JSON）
#[derive(Debug, Serialize)]
pub(super) struct RegisterPayload<'a> {
    pub(super) ed25519_pub: &'a str,
    pub(super) x25519_pub: &'a str,
    pub(super) deviceid: &'a str,
    pub(super) timestamp: u64,
    pub(super) nonce: &'a str,
}

/// 登录载荷（content 解密后的明文 JSON）
#[derive(Debug, Serialize)]
pub(super) struct LoginPayload<'a> {
    pub(super) device_pk: &'a str,
    pub(super) timestamp: u64,
    pub(super) nonce: &'a str,
}

/// 刷新载荷（content 解密后的明文 JSON）
///
/// 与 `LoginPayload` 字段一致，额外携带 `refresh_token`。
/// refresh_token 不在请求体明文中传输，仅出现在 AES-256-GCM 加密的 content 内。
#[derive(Debug, Serialize)]
pub(super) struct RefreshPayload<'a> {
    pub(super) device_pk: &'a str,
    pub(super) timestamp: u64,
    pub(super) nonce: &'a str,
    pub(super) refresh_token: &'a str,
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

/// 协议版本
pub const PROTOCOL_VERSION: &str = "MoSign-v1";

/// HKDF info for session key（与服务端约定）
pub(super) const SESSION_KEY_INFO: &[u8] = b"mosign-v1-session-key";

/// refresh_token 有效期（30 天，秒）
pub const REFRESH_TOKEN_TTL_SECS: u64 = 30 * 24 * 3600;
