//! yggdrasil 认证端点：authenticate / validate / refresh

use super::types::YggdrasilError;
use super::{join_url, parse_error};
use crate::minecraft::auth::authlib::types::{
    AuthResponse, AuthenticateRequest, ProfileId, RefreshRequest, ValidateRequest,
};

/// POST /authserver/authenticate 账号密码登录
pub async fn authenticate(
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<AuthResponse, YggdrasilError> {
    let url = join_url(server_url, "/authserver/authenticate");
    let body = AuthenticateRequest::new(username.to_string(), password.to_string());
    post_json(&url, &body).await
}

/// POST /authserver/validate 校验令牌
///
/// 成功返回 Ok(())，失败返回 Err。
pub async fn validate(
    server_url: &str,
    access_token: &str,
    client_token: Option<&str>,
) -> Result<(), YggdrasilError> {
    let url = join_url(server_url, "/authserver/validate");
    let body = ValidateRequest {
        access_token: access_token.to_string(),
        client_token: client_token.map(|s| s.to_string()),
    };
    // validate 成功返回 204 No Content，与 authenticate/refresh 的 200 不同
    let (status, text) = crate::http::post_json_with_status(&url, &body)
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: e.to_string(),
            is_network: true,
        })?;
    if status == 204 {
        return Ok(());
    }
    Err(parse_error(status, text))
}

/// POST /authserver/refresh 刷新令牌
///
/// 可指定 `selected_profile` 以切换角色（多角色场景）。
pub async fn refresh(
    server_url: &str,
    access_token: &str,
    client_token: Option<&str>,
    selected_profile: Option<ProfileId>,
) -> Result<AuthResponse, YggdrasilError> {
    let url = join_url(server_url, "/authserver/refresh");
    let body = RefreshRequest {
        access_token: access_token.to_string(),
        client_token: client_token.map(|s| s.to_string()),
        selected_profile,
        request_user: true,
    };
    post_json(&url, &body).await
}

/// 通用 POST JSON 请求（用于 authenticate / refresh）
///
/// 通过 `crate::http::post_json_with_status` 发起请求，保留状态码用于差异化错误处理。
async fn post_json<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<AuthResponse, YggdrasilError> {
    let (status, text) = crate::http::post_json_with_status(url, body)
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: e.to_string(),
            is_network: true,
        })?;
    if status == 200 {
        serde_json::from_str::<AuthResponse>(&text).map_err(|e| YggdrasilError {
            status,
            message: format!("解析响应失败: {}", e),
            is_network: false,
        })
    } else {
        Err(parse_error(status, text))
    }
}
