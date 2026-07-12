//! 微软登录模块
//!
//! 实现 Device Code Flow 和 Web Auth Code Flow 的完整 Token 交换链。
//! 根据 Client ID 自动选择流程和端点（参考 PCL2）。

pub mod config;
pub mod exchange;
pub mod oauth;
pub mod types;

// 重导出常用 API
pub use config::{is_official_client, OAUTH_CLIENT_ID};
pub use exchange::{complete_login_chain, get_poll_interval, is_token_expired};
pub use oauth::{
    build_auth_url, exchange_auth_code, poll_device_code, refresh_oauth_token, request_device_code,
};
pub use types::*;

use crate::log_info;

/// 使用 Refresh Token 完成静默刷新（无需用户交互）
///
/// 流程：刷新 OAuth Token → XBL → XSTS → MC Token → 验证 → 档案
pub async fn login_with_refresh_token<F>(
    refresh_token: &str,
    mut progress: F,
) -> Result<MicrosoftLoginResult, MicrosoftLoginError>
where
    F: FnMut(&str),
{
    log_info!("Attempting silent login with refresh token");

    progress("refresh");
    let oauth_response = refresh_oauth_token(refresh_token).await?;

    let new_refresh = oauth_response
        .refresh_token
        .as_deref()
        .unwrap_or(refresh_token);

    complete_login_chain(&oauth_response.access_token, new_refresh, progress).await
}
