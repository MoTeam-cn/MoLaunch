//! 认证命令模块
//!
//! 支持离线登录、微软登录（Web Auth Code Flow / Device Code Flow）和
//! authlib-injector 外置登录（yggdrasil 协议）。
//! 流程选择由 Client ID 自动决定：
//! - 官方 ID（默认）：Web Auth Code Flow + login.live.com 旧版端点
//! - 自定义 ID：Device Code Flow + login.microsoftonline.com v2.0 端点
//!
//! 注：原 23 个分散的 auth Tauri 命令已聚合为 `meta_manager` 一个 IPC 入口，
//! 通过请求体的 `action` 字段分发到各子模块函数。
//! 子模块函数已去掉 `#[tauri::command]` 标注，改为接收 `&AppState` / `&AppHandle`，
//! 由 `utils::meta_manager::dispatch` 反序列化参数后调用。
//! `MetaRequest` 已替换为通用的 `utils::dispatcher::ActionRequest`，
//! 与 `tools_manager` 共用同一请求体结构。

pub mod account;
pub mod authlib;
pub mod microsoft;
pub mod offline;

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// 统一认证 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::meta_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn meta_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::meta_manager::dispatch(state, app, req).await
}

// 重导出所有子模块函数（pub use 仅供普通 Rust 代码调用，
// Tauri 命令注册只通过上面的 meta_manager 单一入口）
pub use account::ms::{get_ms_accounts, remove_ms_account, switch_ms_account};
pub use account::offline::{
    get_offline_accounts, remove_offline_account, save_custom_skin, set_offline_skin,
    switch_offline_account,
};
pub use account::session::{get_login_status, logout};
pub use authlib::{
    authlib_fetch_server_meta, authlib_login, authlib_select_profile, get_authlib_accounts,
    remove_authlib_account, switch_authlib_account, AuthlibAccountInfo, AuthlibLoginResult,
    AuthlibServerMeta, PendingAuthlibLogin,
};
pub use microsoft::{
    ms_login_get_config, ms_login_poll, ms_login_refresh, ms_login_request_device_code,
    ms_login_web_exchange, ms_login_web_start, DeviceCodeInfo, LoginConfig, PollResult,
};
pub use offline::login_offline;
