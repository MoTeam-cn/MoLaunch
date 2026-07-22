//! 认证命令模块
//!
//! 支持离线登录和微软登录（Web Auth Code Flow / Device Code Flow）。
//! 流程选择由 Client ID 自动决定：
//! - 官方 ID（默认）：Web Auth Code Flow + login.live.com 旧版端点
//! - 自定义 ID：Device Code Flow + login.microsoftonline.com v2.0 端点

pub mod account;
pub mod microsoft;
pub mod offline;

// 重导出所有 Tauri 命令（命令分散到 account 的 ms/offline/session 子模块）
// 注意：tauri::command 宏的 __cmd__ 符号无法通过 pub use 重导出，
// lib.rs 使用完整路径注册（commands::auth::account::ms::* / ::offline::* / ::session::*），
// 此处的 pub use 仅供普通 Rust 代码调用使用
pub use account::ms::{get_ms_accounts, remove_ms_account, switch_ms_account};
pub use account::offline::{
    get_offline_accounts, remove_offline_account, save_custom_skin, set_offline_skin,
    switch_offline_account,
};
pub use account::session::{get_login_status, logout};
pub use microsoft::{
    ms_login_get_config, ms_login_poll, ms_login_refresh, ms_login_request_device_code,
    ms_login_web_exchange, ms_login_web_start,
};
pub use offline::login_offline;
