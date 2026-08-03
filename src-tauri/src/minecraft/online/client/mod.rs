//! api-server HTTP 客户端封装
//! 提供与 MoLaunch API Server 交互的统一入口。
//! 子模块：auth（认证）、time（时间同步）、jwks（JWKS/CSRF）、request（业务请求）。

// 重导出类型供外部模块使用（signaling.rs 等 `use super::client::{BusinessResult, ClientError, OnlineClient}`）
pub use super::client_types::{BusinessResult, ClientError};

mod auth;
mod core;
mod jwks;
mod request;
mod time;

pub use core::OnlineClient;

#[cfg(test)]
#[path = "../client_tests.rs"]
mod tests;
