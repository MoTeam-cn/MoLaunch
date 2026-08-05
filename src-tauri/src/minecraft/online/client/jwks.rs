//! JWKS / CSRF 接口：JWKS 拉取、RSA 公钥提取、CSRF Token 获取。

use crate::api_paths;
use crate::http::get_client;
use crate::minecraft::online::client_types::{ClientError, CsrfResponse, JwkKey, JwksResponse};
use crate::minecraft::online::crypto::b64u_decode;
use crate::minecraft::online::http_log;

use super::OnlineClient;

impl OnlineClient {
    /// 获取 JWKS（GET /v3/.well-known/jwks.json）
    ///
    /// 仅模块内调用（`verify_jwt` 阶段二实现），不对外暴露。
    async fn get_jwks(&self) -> Result<Vec<JwkKey>, ClientError> {
        let url = format!("{}{}", self.base_url, api_paths::JWKS_JSON);
        crate::log_debug!("[Online] GET {}", url);
        let resp = get_client().get(&url).send().await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        http_log::log_http_request(
            "GET",
            api_paths::JWKS_JSON,
            status,
            &http_log::extract_req_id(&body),
        );
        crate::log_debug!(
            "[Online] {} 响应 status={}, body_len={}",
            api_paths::JWKS_JSON,
            status,
            body.len()
        );
        if status != 200 {
            crate::log_warn!("[Online] JWKS 非 200: status={}, body={}", status, body);
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
        let url = format!("{}{}", self.base_url, api_paths::CSRF_TOKEN);
        let resp = get_client()
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .send()
            .await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        http_log::log_http_request(
            "GET",
            api_paths::CSRF_TOKEN,
            status,
            &http_log::extract_req_id(&body),
        );
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
}
