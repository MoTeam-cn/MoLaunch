//! 认证命令模块
//!
//! 支持离线登录和微软登录（Web Auth Code Flow / Device Code Flow）。
//! 流程选择由 Client ID 自动决定：
//! - 官方 ID（默认）：Web Auth Code Flow + login.live.com 旧版端点
//! - 自定义 ID：Device Code Flow + login.microsoftonline.com v2.0 端点

pub mod account;
pub mod microsoft;
pub mod offline;

// 重导出所有 Tauri 命令
pub use account::{
    get_login_status, get_ms_accounts, get_offline_accounts, logout, remove_ms_account,
    remove_offline_account, set_offline_skin, switch_ms_account, switch_offline_account,
};
pub use microsoft::{
    ms_login_get_config, ms_login_poll, ms_login_refresh, ms_login_request_device_code,
    ms_login_web_exchange, ms_login_web_start,
};
pub use offline::login_offline;
