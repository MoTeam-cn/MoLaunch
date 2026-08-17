//! MoSign-v1 设备认证协议
//!
//! 实现 MoLaunch API Server 的设备注册/登录/登出流程。

mod helpers;
mod keypair;
mod login;
mod refresh;
mod register;
mod types;

pub use helpers::generate_device_id;
pub use keypair::OnlineKeyPair;
pub use login::{build_login_request, finalize_credentials_with_login};
pub use refresh::{build_refresh_request, finalize_credentials_with_refresh};
pub use register::{build_register_request, finalize_credentials_with_register};
pub use types::{
    LoginData, LoginRequest, LoginResponse, RefreshData, RefreshRequest, RefreshResponse,
    RegisterData, RegisterRequest, RegisterResponse, PROTOCOL_VERSION, REFRESH_TOKEN_TTL_SECS,
};

// 私有 use：让 login/refresh 子模块的 `super::SESSION_KEY_INFO` 保持可用
use types::SESSION_KEY_INFO;

// 测试用：跨模块符号引入父模块命名空间（测试 `use super::*;` 可访问）
#[cfg(test)]
use super::crypto::b64u_encode;
#[cfg(test)]
use super::storage::DeviceCredentials;

#[cfg(test)]
#[path = "../auth_tests.rs"]
mod tests;
