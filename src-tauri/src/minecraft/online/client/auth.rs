//! 认证接口：注册/登录/登出/刷新（POST /v3/auth/*）

use crate::api_paths;
use crate::http::get_client;
use crate::minecraft::online::auth::{
    LoginRequest, LoginResponse, RefreshRequest, RefreshResponse, RegisterRequest, RegisterResponse,
};
use crate::minecraft::online::client_types::ClientError;
use crate::minecraft::online::http_log;

use super::OnlineClient;

impl OnlineClient {
    /// 注册设备（POST /v3/auth/register）
    pub async fn register(&self, req: &RegisterRequest) -> Result<RegisterResponse, ClientError> {
        let url = format!("{}{}", self.base_url, api_paths::AUTH_REGISTER);
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
        http_log::log_http_request(
            "POST",
            api_paths::AUTH_REGISTER,
            status,
            &http_log::extract_req_id(&body),
        );
        crate::log_info!(
            "[Online] {} 响应 status={}, body_len={}",
            api_paths::AUTH_REGISTER,
            status,
            body.len()
        );
        if status != 200 && status != 400 {
            crate::log_error!("[Online] 注册 HTTP 异常: status={}, body={}", status, body);
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
        let url = format!("{}{}", self.base_url, api_paths::AUTH_LOGIN);
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
        http_log::log_http_request(
            "POST",
            api_paths::AUTH_LOGIN,
            status,
            &http_log::extract_req_id(&body),
        );
        crate::log_info!(
            "[Online] {} 响应 status={}, body_len={}",
            api_paths::AUTH_LOGIN,
            status,
            body.len()
        );
        if status != 200 && status != 400 {
            crate::log_error!("[Online] 登录 HTTP 异常: status={}, body={}", status, body);
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
        let url = format!("{}{}", self.base_url, api_paths::AUTH_LOGOUT);
        crate::log_info!("[Online] POST {}", url);
        let resp = get_client()
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .send()
            .await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        http_log::log_http_request(
            "POST",
            api_paths::AUTH_LOGOUT,
            status,
            &http_log::extract_req_id(&body),
        );
        crate::log_info!(
            "[Online] {} 响应 status={}, body_len={}",
            api_paths::AUTH_LOGOUT,
            status,
            body.len()
        );
        if status != 200 && status != 401 {
            crate::log_error!("[Online] 登出 HTTP 异常: status={}, body={}", status, body);
            return Err(ClientError::HttpStatus { status, body });
        }
        Ok(())
    }

    /// 续期 access token（POST /v3/auth/refresh）
    ///
    /// 采用与登录一致的 MoSign-v1 协议：ECDH 派生会话密钥 + AES-256-GCM 加密 content +
    /// HMAC-SHA256 签名。`refresh_token` 放在加密的 content 内，明文不出现在请求体。
    /// 服务端会轮换 refresh_token（旧 refresh_token 用后失效）。
    pub async fn refresh(&self, req: &RefreshRequest) -> Result<RefreshResponse, ClientError> {
        let url = format!("{}{}", self.base_url, api_paths::AUTH_REFRESH);
        crate::log_info!(
            "[Online] POST {} (device_pk={}, content_len={})",
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
        http_log::log_http_request(
            "POST",
            api_paths::AUTH_REFRESH,
            status,
            &http_log::extract_req_id(&body),
        );
        crate::log_info!(
            "[Online] {} 响应 status={}, body_len={}",
            api_paths::AUTH_REFRESH,
            status,
            body.len()
        );
        if status != 200 && status != 400 && status != 401 {
            crate::log_error!(
                "[Online] refresh HTTP 异常: status={}, body={}",
                status,
                body
            );
            return Err(ClientError::HttpStatus { status, body });
        }
        let parsed: RefreshResponse = serde_json::from_str(&body)?;
        if parsed.code != 1 {
            crate::log_warn!(
                "[Online] refresh 业务失败: code={}, msg={}",
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
}
