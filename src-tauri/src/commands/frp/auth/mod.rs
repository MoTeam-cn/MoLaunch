//! Frp 厂商认证模块：OAuth2 / Device Code / API Key 三种流程
//!
//! token 经 SDK 内置 DES 加密后存文件（`<base_dir>/frp/auth/{provider_id}.json`）。
//! 子模块：storage（加密存储辅助）/ oauth2（exchange+flow）/ device_code / api_key /
//! flows（可配置流程引擎）/ handlers（公开 API 处理函数）/ types（返回类型）。

mod api_key;
mod device_code;
mod flows;
mod handlers;
mod oauth2;
mod pkce;
mod storage;
mod types;

pub use api_key::save_api_key;
pub use device_code::{poll_device_code, start_device_code};
pub use handlers::{ensure_valid_token, get_auth_status, load_token, refresh_token, revoke_auth};
pub use oauth2::start_oauth2;
pub use storage::set_sdk;
pub use types::{AuthStatus, DeviceCodePollResult, DeviceCodeResult, OAuth2Result};
