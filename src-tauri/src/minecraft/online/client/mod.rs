//! api-server HTTP 客户端封装
//! 提供与 MoLaunch API Server 交互的统一入口。
//! 子模块：auth（认证）、time（时间同步）、jwks（JWKS/CSRF）、request（业务请求）。

// 重导出类型供外部模块使用（signaling.rs 等 `use super::client::{BusinessResult, ClientError, OnlineClient}`）
pub use super::client_types::{BusinessResult, ClientError};

mod auth;
mod jwks;
mod request;
mod time;

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
}

#[cfg(test)]
#[path = "../client_tests.rs"]
mod tests;
