//! api-server 客户端类型定义
//!
//! 响应模型与错误类型，从 `client.rs` 拆出以控制文件行数。
//! `client.rs` 通过 `pub use` 重导出，外部模块（如 `signaling.rs`）的
//! `use super::client::{BusinessResult, ClientError, OnlineClient}` 无需改动。

use serde::{Deserialize, Serialize};

use super::crypto::{b64u_decode, CryptoError};

/// 统一响应格式
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnifiedResponse<T = serde_json::Value> {
    pub code: u32,
    pub data: Option<T>,
    pub msg: String,
    #[serde(default)]
    pub time: String,
    #[serde(default)]
    pub req_id: String,
}

/// JWKS 公钥
///
/// `kid` / `alg` / `use_` 字段为 JWKS 规范标准字段，当前未参与校验，
/// 阶段二接入 JWT 签名验证时会用于匹配 `kid` 与算法。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct JwkKey {
    pub kty: String,
    pub kid: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub n: String,
    pub e: String,
}

impl JwkKey {
    /// 将 JWKS 的 (n, e) 转换为 PEM SPKI 格式
    ///
    /// 用于注册时传给 `rsa_oaep_encrypt`。
    pub fn to_pem(&self) -> Result<String, ClientError> {
        use rsa::pkcs8::EncodePublicKey;
        use rsa::{BigUint, RsaPublicKey};

        let n_bytes = b64u_decode(&self.n)?;
        let e_bytes = b64u_decode(&self.e)?;
        let n = BigUint::from_bytes_be(&n_bytes);
        let e = BigUint::from_bytes_be(&e_bytes);
        let pub_key = RsaPublicKey::new(n, e)
            .map_err(|e| ClientError::RsaRebuildFailed(e.to_string()))?;
        pub_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| ClientError::RsaRebuildFailed(e.to_string()))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwksResponse {
    pub code: u32,
    pub data: Option<JwksData>,
    pub msg: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwksData {
    pub keys: Vec<JwkKey>,
}

/// CSRF Token 响应
#[derive(Debug, Clone, Deserialize)]
pub struct CsrfResponse {
    pub code: u32,
    pub data: Option<CsrfData>,
    pub msg: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrfData {
    pub token: String,
}

/// 时间校准响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeResponse {
    pub code: u32,
    pub data: Option<TimeData>,
    pub msg: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeData {
    pub server_time: u64,
    pub rfc3339: String,
    pub timezone: String,
    pub offset_seconds: i32,
}

/// 业务接口调用结果（解密后）
#[derive(Debug, Clone, Serialize)]
pub struct BusinessResult<T> {
    pub code: u32,
    pub data: Option<T>,
    pub msg: String,
    pub req_id: String,
}

/// 客户端错误
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),

    #[error("HTTP {status}: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),

    #[error("加密错误: {0}")]
    Crypto(#[from] CryptoError),

    #[error("业务错误 [{code}]: {msg}")]
    Business { code: u32, msg: String },

    /// 服务端返回 code=1003（未授权）：token 被撤销或 RSA 密钥变更
    ///
    /// Display 仅展示 msg（req_id 由 HTTP 日志记录，用户可自行翻阅，无需弹窗显示）
    #[error("未授权 (code=1003): {msg}")]
    Unauthorized { msg: String, req_id: String },

    #[error("设备未注册或凭证缺失")]
    NotRegistered,

    #[error("JWT 已过期")]
    TokenExpired,

    #[error("JWKS 中找不到 kid={0} 的公钥")]
    JwksKidNotFound(String),

    #[error("RSA 公钥重建失败: {0}")]
    RsaRebuildFailed(String),

    #[error("响应不是 ECIES 加密信封（明文响应）: {0}")]
    NotEnvelope(String),
}
