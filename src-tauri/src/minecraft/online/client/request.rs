//! 业务请求封装：/v1 接口统一调用（ECIES 信封 + JWT + CSRF + 业务结果解析）

use crate::http::get_client;
use crate::minecraft::online::client_types::{BusinessResult, ClientError, UnifiedResponse};
use crate::minecraft::online::crypto::{b64u_decode, CryptoError};
use crate::minecraft::online::ecies::{is_envelope, open, seal, Envelope};
use crate::minecraft::online::http_log;
use crate::minecraft::online::pow::{parse_challenge, solve_challenge};
use crate::minecraft::online::storage::DeviceCredentials;

use super::OnlineClient;

impl OnlineClient {
    /// 发送请求并自动处理服务端 PoW challenge 重试（auth 鉴权与 /v1 业务请求共用）
    ///
    /// `build_request(pow_proof)` 每次构造请求：首次不携带 PoW 头；收到
    /// `401 + code:1007` 且 challenge.path 与 path_label 一致时，求解后携带
    /// `{header_name}: {challenge_id}:{nonce}` 头重试一次。求解超时/失败按
    /// 原始 401 返回，交由调用方处理。
    pub(super) async fn send_with_pow_retry(
        path_label: &str,
        mut build_request: impl FnMut(Option<&(String, String)>) -> reqwest::RequestBuilder,
    ) -> Result<(u16, String), ClientError> {
        let mut pow_proof: Option<(String, String)> = None;
        loop {
            let req = build_request(pow_proof.as_ref());
            let resp = req.send().await?;
            let status = resp.status().as_u16();
            let body = resp.text().await?;

            if status == 401 && pow_proof.is_none() {
                if let Some(challenge) = parse_challenge(&body) {
                    if challenge.path == path_label {
                        if let Some(salt) = challenge.salt_bytes() {
                            if let Some(nonce) = solve_challenge(&salt, challenge.difficulty).await
                            {
                                crate::log_info!(
                                    "[Online] {} PoW 求解成功（difficulty={}, nonce={}）",
                                    path_label,
                                    challenge.difficulty,
                                    nonce
                                );
                                pow_proof = Some((
                                    challenge.header_name.clone(),
                                    format!("{}:{}", challenge.challenge_id, nonce),
                                ));
                                continue;
                            }
                        }
                    }
                }
                crate::log_warn!("[Online] {} PoW 求解失败/放弃，按原始 401 处理", path_label);
            }
            return Ok((status, body));
        }
    }

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
            crate::log_debug!("[Online] 请求体明文长度={}B, 加密中...", plaintext.len());
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

        // 发起请求（自动处理服务端 PoW challenge：401 + code:1007 时求解后重试一次）
        let method_upper = method.to_uppercase();
        if !matches!(
            method_upper.as_str(),
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
        ) {
            return Err(ClientError::Business {
                code: 0,
                msg: format!("不支持的 HTTP 方法: {}", method_upper),
            });
        }

        let (status, body_text) = Self::send_with_pow_retry(path, |pow_proof| {
            let req_builder = match method_upper.as_str() {
                "GET" => get_client().get(&url),
                "POST" => get_client().post(&url),
                "PUT" => get_client().put(&url),
                "PATCH" => get_client().patch(&url),
                "DELETE" => get_client().delete(&url),
                _ => unreachable!("method 已在调用前校验"),
            };
            let mut req_builder = req_builder.header("Authorization", format!("Bearer {}", jwt));

            if need_csrf {
                req_builder = req_builder.header("X-CSRF-Token", &csrf_token);
            }

            if let Some((header_name, proof)) = pow_proof {
                req_builder = req_builder.header(header_name, proof);
            }

            if let Some(envelope) = &req_body {
                req_builder = req_builder
                    .header("Content-Type", "application/json")
                    .json(envelope);
            }

            req_builder
        })
        .await?;

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
        //
        // HTTP 请求日志的 req_id 从解密后的 unified 响应体提取（加密信封的
        // body_text 不含 req_id 字段），故日志记录移至解密完成后统一执行。
        // JSON 解析失败时用 extract_req_id 兜底（明文错误响应可能含 req_id）。
        let body_json: serde_json::Value = match serde_json::from_str(&body_text) {
            Ok(v) => v,
            Err(_) => {
                crate::log_error!(
                    "[Online] call_v1 响应 JSON 解析失败: {} {} body_len={}B",
                    method,
                    path,
                    body_text.len()
                );
                let fallback_req_id = http_log::extract_req_id(&body_text);
                http_log::log_http_request(method, path, status, &fallback_req_id);
                return Err(ClientError::HttpStatus {
                    status,
                    body: body_text.clone(),
                });
            }
        };

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

            // 记录 HTTP 请求日志（加密分支：req_id 从解密后的 unified 提取）
            http_log::log_http_request(method, path, status, &unified.req_id);

            // code=1003 表示未授权（token 被撤销或 RSA 密钥变更），返回错误以便上层处理
            if unified.code == 1003 {
                return Err(ClientError::Unauthorized {
                    msg: unified.msg,
                    req_id: unified.req_id,
                });
            }

            Ok(BusinessResult {
                code: unified.code,
                data: unified.data,
                msg: unified.msg,
                req_id: unified.req_id,
            })
        } else {
            // 明文响应（401/400/500）
            crate::log_debug!("[Online] call_v1 响应为明文: {} {}", method, path);
            let unified: UnifiedResponse<T> = serde_json::from_value(body_json)?;
            crate::log_warn!(
                "[Online] call_v1 明文响应: {} {} code={}, msg={}, req_id={}",
                method,
                path,
                unified.code,
                unified.msg,
                unified.req_id
            );

            // 记录 HTTP 请求日志（明文分支：req_id 从 unified 提取）
            http_log::log_http_request(method, path, status, &unified.req_id);

            // code=1003 表示未授权（token 被撤销或 RSA 密钥变更），返回错误以便上层处理
            if unified.code == 1003 {
                return Err(ClientError::Unauthorized {
                    msg: unified.msg,
                    req_id: unified.req_id,
                });
            }

            Ok(BusinessResult {
                code: unified.code,
                data: unified.data,
                msg: unified.msg,
                req_id: unified.req_id,
            })
        }
    }
}
