//! 微软登录模块
//!
//! 实现 Device Code Flow 和 Web Auth Code Flow 的完整 Token 交换链。
//! 根据 Client ID 自动选择流程和端点。

pub mod config;
pub mod exchange;
pub mod oauth;
pub mod types;

// 重导出常用 API
pub use config::{is_official_client, OAUTH_CLIENT_ID};
pub use exchange::{
    complete_login_chain, get_poll_interval, is_token_expired, login_with_refresh_token,
};
pub use oauth::{
    build_auth_url, exchange_auth_code, poll_device_code, refresh_oauth_token, request_device_code,
};
pub use types::*;
