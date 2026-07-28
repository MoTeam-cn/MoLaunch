//! api-server HTTP 客户端封装
//!
//! 提供与 MoLaunch API Server 交互的统一入口：
//! - `/v3/*` 认证接口：注册/登录/登出、JWKS、CSRF、时间校准
//! - `/v1/*` 业务接口：自动加 ECIES 信封、JWT 携带、CSRF 校验
//!
//! 接口参考：`api-server/docs/auth.md`、`api-server/docs/signaling.md`
//!
//! 类型与错误定义见 `client_types.rs`，本文件通过 `pub use` 重导出，
//! 外部模块（如 `signaling.rs`）的 `use super::client::{BusinessResult, ClientError, OnlineClient}` 无需改动。

use super::auth::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use super::client_types::{CsrfResponse, JwkKey, JwksResponse, TimeData, TimeResponse, UnifiedResponse};
use super::ecies::{is_envelope, open, seal, Envelope};
use super::storage::DeviceCredentials;
use crate::http::get_client;
use crate::minecraft::online::crypto::{b64u_decode, CryptoError};

// 重导出类型供外部模块使用（signaling.rs 等 `use super::client::{BusinessResult, ClientError, OnlineClient}`）
pub use super::client_types::{BusinessResult, ClientError};

/// api-server 客户端
pub struct OnlineClient {
    base_url: String,
}

impl OnlineClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// 更新 base_url（用户在设置页修改 api-server 地址后调用）
    pub fn update_base_url(&mut self, base_url: &str) {
        self.base_url = base_url.trim_end_matches('/').to_string();
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // ============================== /v3 认证接口 ==============================

    /// 获取服务器时间（GET /v3/time）
    pub async fn get_server_time(&self) -> Result<TimeData, ClientError> {
        let url = format!("{}/v3/time", self.base_url);
        crate::log_debug!("[Online] GET {}", url);
        let resp = get_client().get(&url).send().await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        crate::log_debug!(
            "[Online] /v3/time 响应 status={}, body_len={}",
            status,
            body.len()
        );
        if status != 200 {
            crate::log_warn!(
                "[Online] /v3/time 非 200: status={}, body={}",
                status,
                body
            );
            return Err(ClientError::HttpStatus { status, body });
        }
        let parsed: TimeResponse = serde_json::from_str(&body)?;
        parsed.data.ok_or_else(|| ClientError::Business {
            code: parsed.code,
            msg: parsed.msg,
        })
    }

    /// 获取 JWKS（GET /v3/.well-known/jwks.json）
    ///
    /// 仅模块内调用（`verify_jwt` 阶段二实现），不对外暴露。
    async fn get_jwks(&self) -> Result<Vec<JwkKey>, ClientError> {
        let url = format!("{}/v3/.well-known/jwks.json", self.base_url);
        crate::log_debug!("[Online] GET {}", url);
        let resp = get_client().get(&url).send().await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        crate::log_debug!(
            "[Online] /v3/.well-known/jwks.json 响应 status={}, body_len={}",
            status,
            body.len()
        );
        if status != 200 {
            crate::log_warn!(
                "[Online] JWKS 非 200: status={}, body={}",
                status,
                body
            );
            return Err(ClientError::HttpStatus { status, body });
        }
        let parsed: JwksResponse = serde_json::from_str(&body)?;
        let data = parsed.data.ok_or_else(|| ClientError::Business {
            code: parsed.code,
            msg: parsed.msg,
        })?;
        crate::log_debug!("[Online] JWKS 返回 {} 个 key", data.keys.len());
        Ok(data.keys)
    }

    /// 从 JWKS 中提取 RSA 公钥 PEM（用于注册时 RSA-OAEP 加密）
    ///
    /// 目前 api-server 只有一个公钥（kid="molaunch-api-1"），返回第一个 RSA key。
    pub async fn fetch_server_rsa_pem(&self) -> Result<String, ClientError> {
        let keys = self.get_jwks().await?;
        let key = keys
            .into_iter()
            .find(|k| k.kty == "RSA")
            .ok_or_else(|| ClientError::JwksKidNotFound("RSA".to_string()))?;
        // 从 JWK n 字段直接计算 RSA 位数（避免再次解析 PEM）
        let n_bytes = b64u_decode(&key.n)?;
        let key_bits = n_bytes.len() * 8;
        crate::log_info!(
            "[Online] 云端 RSA 公钥已获取: kid={}, 位数={}bit (modulus={}B)",
            key.kid,
            key_bits,
            n_bytes.len()
        );
        if key_bits < 3072 {
            crate::log_warn!(
                "[Online] 云端 RSA 公钥位数仅 {}bit，注册 content 约 209B，\
                 RSA-2048 + OAEP-SHA256 最大允许 190B，将触发 \"message too long\" 错误。\
                 请在 api-server 端重新生成 RSA-3072 或 RSA-4096 密钥。",
                key_bits
            );
        }
        key.to_pem()
    }

    /// 获取 CSRF Token（GET /v3/csrf/token）
    ///
    /// 调用 /v1 非幂等接口（POST/DELETE）前需先获取。
    pub async fn get_csrf_token(&self, jwt: &str) -> Result<String, ClientError> {
        let url = format!("{}/v3/csrf/token", self.base_url);
        let resp = get_client()
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .send()
            .await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        if status != 200 {
            return Err(ClientError::HttpStatus { status, body });
        }
        let parsed: CsrfResponse = serde_json::from_str(&body)?;
        let data = parsed.data.ok_or_else(|| ClientError::Business {
            code: parsed.code,
            msg: parsed.msg,
        })?;
        Ok(data.token)
    }

    /// 注册设备（POST /v3/auth/register）
    pub async fn register(&self, req: &RegisterRequest) -> Result<RegisterResponse, ClientError> {
        let url = format!("{}/v3/auth/register", self.base_url);
        crate::log_info!(
            "[Online] POST {} (deviceid={}, content_len={}B)",
            url,
            req.deviceid,
            req.content.len()
        );
        let resp = get_client()
            .post(&url)
            .header("Content-Type", "application/json")
            .json(req)
            .send()
            .await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        crate::log_info!(
            "[Online] /v3/auth/register 响应 status={}, body_len={}",
            status,
            body.len()
        );
        if status != 200 && status != 400 {
            crate::log_error!(
                "[Online] 注册 HTTP 异常: status={}, body={}",
                status,
                body
            );
            return Err(ClientError::HttpStatus { status, body });
        }
        let parsed: RegisterResponse = serde_json::from_str(&body)?;
        if parsed.code != 1 {
            crate::log_warn!(
                "[Online] 注册业务失败: code={}, msg={}",
                parsed.code,
                parsed.msg
            );
            return Err(ClientError::Business {
                code: parsed.code,
                msg: parsed.msg,
            });
        }
        Ok(parsed)
    }

    /// 登录设备（POST /v3/auth/login）
    pub async fn login(&self, req: &LoginRequest) -> Result<LoginResponse, ClientError> {
        let url = format!("{}/v3/auth/login", self.base_url);
        crate::log_info!(
            "[Online] POST {} (device_pk={}, content_len={}B)",
            url,
            req.device_pk,
            req.content.len()
        );
        let resp = get_client()
            .post(&url)
            .header("Content-Type", "application/json")
            .json(req)
            .send()
            .await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        crate::log_info!(
            "[Online] /v3/auth/login 响应 status={}, body_len={}",
            status,
            body.len()
        );
        if status != 200 && status != 400 {
            crate::log_error!(
                "[Online] 登录 HTTP 异常: status={}, body={}",
                status,
                body
            );
            return Err(ClientError::HttpStatus { status, body });
        }
        let parsed: LoginResponse = serde_json::from_str(&body)?;
        if parsed.code != 1 {
            crate::log_warn!(
                "[Online] 登录业务失败: code={}, msg={}",
                parsed.code,
                parsed.msg
            );
            return Err(ClientError::Business {
                code: parsed.code,
                msg: parsed.msg,
            });
        }
        Ok(parsed)
    }

    /// 登出（POST /v3/auth/logout）
    pub async fn logout(&self, jwt: &str) -> Result<(), ClientError> {
        let url = format!("{}/v3/auth/logout", self.base_url);
        crate::log_info!("[Online] POST {}", url);
        let resp = get_client()
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .send()
            .await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        crate::log_info!(
            "[Online] /v3/auth/logout 响应 status={}, body_len={}",
            status,
            body.len()
        );
        if status != 200 && status != 401 {
            crate::log_error!(
                "[Online] 登出 HTTP 异常: status={}, body={}",
                status,
                body
            );
            return Err(ClientError::HttpStatus { status, body });
        }
        Ok(())
    }

    // ============================== /v1 业务接口 ==============================

    /// 调用 /v1 业务接口（自动加 ECIES 信封、携带 JWT、CSRF）
    ///
    /// 参数：
    /// - `creds`：设备凭证（含 JWT、device_public_key、X25519 私钥）
    /// - `method`：HTTP 方法（"GET" / "POST" / "PUT" / "PATCH" / "DELETE"）
    /// - `path`：接口路径（如 "/v1/signaling/stun"）
    /// - `body`：请求体明文（GET 传 None）
    /// - `need_csrf`：是否需要 CSRF token（POST/PUT/PATCH/DELETE 需要，GET 不需要）
    pub async fn call_v1<T: serde::de::DeserializeOwned>(
        &self,
        creds: &DeviceCredentials,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
        need_csrf: bool,
    ) -> Result<BusinessResult<T>, ClientError> {
        if !creds.is_registered() {
            return Err(ClientError::NotRegistered);
        }
        if creds.is_token_expired() {
            return Err(ClientError::TokenExpired);
        }

        let url = format!("{}{}", self.base_url, path);
        let jwt = &creds.device_token;

        crate::log_debug!(
            "[Online] call_v1 开始: {} {} (csrf={}, body={})",
            method,
            path,
            if need_csrf { "yes" } else { "no" },
            if body.is_some() { "yes" } else { "no" },
        );

        // GET 请求需要 CSRF（因为 /v1 全局 CSRF 中间件校验所有非 /v3 请求）
        // 实际上 GET 也是非幂等的（按 api-server 配置），所以统一获取
        let csrf_token = if need_csrf {
            match self.get_csrf_token(jwt).await {
                Ok(t) => {
                    crate::log_debug!("[Online] CSRF 获取成功, len={}", t.len());
                    t
                }
                Err(e) => {
                    crate::log_warn!("[Online] CSRF 获取失败: {}", e);
                    return Err(e);
                }
            }
        } else {
            String::new()
        };

        // 请求体加密（如果有）
        let req_body: Option<Envelope> = if let Some(body_val) = body {
            let plaintext = serde_json::to_vec(body_val)?;
            crate::log_debug!(
                "[Online] 请求体明文长度={}B, 加密中...",
                plaintext.len()
            );
            let sealed = seal(&plaintext, &creds.device_public_key_b64u)?;
            crate::log_debug!(
                "[Online] 请求体加密完成 (payload_len={}, key_len={})",
                sealed.envelope.payload.len(),
                sealed.envelope.key.len()
            );
            Some(sealed.envelope)
        } else {
            None
        };

        // 发起请求
        let req_builder = match method.to_uppercase().as_str() {
            "GET" => get_client().get(&url),
            "POST" => get_client().post(&url),
            "PUT" => get_client().put(&url),
            "PATCH" => get_client().patch(&url),
            "DELETE" => get_client().delete(&url),
            other => {
                return Err(ClientError::Business {
                    code: 0,
                    msg: format!("不支持的 HTTP 方法: {}", other),
                })
            }
        };

        let mut req_builder = req_builder.header("Authorization", format!("Bearer {}", jwt));

        if need_csrf {
            req_builder = req_builder.header("X-CSRF-Token", &csrf_token);
        }

        if let Some(envelope) = &req_body {
            req_builder = req_builder
                .header("Content-Type", "application/json")
                .json(envelope);
        }

        let resp = req_builder.send().await?;
        let status = resp.status().as_u16();
        let body_text = resp.text().await?;

        crate::log_debug!(
            "[Online] call_v1 响应: {} {} status={}, body_len={}B",
            method,
            path,
            status,
            body_text.len()
        );

        if status != 200 {
            crate::log_warn!(
                "[Online] call_v1 非 200: {} {} status={}, body={}",
                method,
                path,
                status,
                body_text
            );
        }

        // 解析响应
        let body_json: serde_json::Value = serde_json::from_str(&body_text).map_err(|_| {
            crate::log_error!(
                "[Online] call_v1 响应 JSON 解析失败: {} {} body_len={}B",
                method,
                path,
                body_text.len()
            );
            ClientError::HttpStatus {
                status,
                body: body_text.clone(),
            }
        })?;

        // 判断是否为加密信封
        if is_envelope(&body_json) {
            crate::log_debug!(
                "[Online] call_v1 响应为加密信封, 解密中: {} {}",
                method,
                path
            );
            let envelope: Envelope = serde_json::from_value(body_json)?;

            // 用本地 X25519 私钥解密
            let secret_bytes_vec = b64u_decode(&creds.x25519_secret_b64u)?;
            if secret_bytes_vec.len() != 32 {
                return Err(ClientError::Crypto(CryptoError::InvalidKeyLength {
                    expected: 32,
                    actual: secret_bytes_vec.len(),
                }));
            }
            let mut secret_bytes = [0u8; 32];
            secret_bytes.copy_from_slice(&secret_bytes_vec);

            let plaintext = open(&envelope, &secret_bytes)?;
            crate::log_debug!(
                "[Online] call_v1 解密成功: {} {} 明文长度={}B",
                method,
                path,
                plaintext.len()
            );
            let unified: UnifiedResponse<T> = serde_json::from_slice(&plaintext)?;

            if unified.code != 1 {
                crate::log_warn!(
                    "[Online] call_v1 业务失败: {} {} code={}, msg={}, req_id={}",
                    method,
                    path,
                    unified.code,
                    unified.msg,
                    unified.req_id
                );
            } else {
                crate::log_debug!(
                    "[Online] call_v1 业务成功: {} {} req_id={}",
                    method,
                    path,
                    unified.req_id
                );
            }

            Ok(BusinessResult {
                code: unified.code,
                data: unified.data,
                msg: unified.msg,
                req_id: unified.req_id,
            })
        } else {
            // 明文响应（401/400/500）
            crate::log_debug!(
                "[Online] call_v1 响应为明文: {} {}",
                method,
                path
            );
            let unified: UnifiedResponse<T> = serde_json::from_value(body_json)?;
            crate::log_warn!(
                "[Online] call_v1 明文响应: {} {} code={}, msg={}, req_id={}",
                method,
                path,
                unified.code,
                unified.msg,
                unified.req_id
            );
            Ok(BusinessResult {
                code: unified.code,
                data: unified.data,
                msg: unified.msg,
                req_id: unified.req_id,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_url_trim() {
        let c1 = OnlineClient::new("https://api.example.com/");
        assert_eq!(c1.base_url(), "https://api.example.com");
        let c2 = OnlineClient::new("https://api.example.com");
        assert_eq!(c2.base_url(), "https://api.example.com");
    }
}
