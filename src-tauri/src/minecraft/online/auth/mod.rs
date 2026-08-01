//! MoSign-v1 设备认证协议
//!
//! 实现 MoLaunch API Server 的设备注册/登录/登出流程。
//! 协议参考：`api-server/docs/auth.md`（注册/登录/刷新的完整流程与算法清单）

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
    RegisterData, RegisterRequest, RegisterResponse,
};

/// 协议版本
pub const PROTOCOL_VERSION: &str = "MoSign-v1";

/// HKDF info for session key（与服务端约定）
const SESSION_KEY_INFO: &[u8] = b"mosign-v1-session-key";

/// refresh_token 有效期（30 天，秒）
pub const REFRESH_TOKEN_TTL_SECS: u64 = 30 * 24 * 3600;

// 测试用：跨模块符号引入父模块命名空间（测试 `use super::*;` 可访问）
#[cfg(test)]
use super::crypto::b64u_encode;
#[cfg(test)]
use super::storage::DeviceCredentials;

#[cfg(test)]
#[path = "../auth_tests.rs"]
mod tests;
