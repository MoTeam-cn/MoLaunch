//! 认证命令模块
//! 支持离线/微软（Web Auth Code Flow / Device Code Flow）/ authlib-injector 外置登录
//! （yggdrasil 协议）。流程选择由 Client ID 自动决定：官方 ID（默认）走 Web Auth Code
//! Flow + login.live.com 旧版端点；自定义 ID 走 Device Code Flow + login.microsoftonline.com v2.0。

pub mod account;
pub mod authlib;
pub(crate) mod meta_manager;
pub mod microsoft;
pub mod offline;

// 重导出所有子模块函数（pub use 仅供普通 Rust 代码调用，
// Tauri 命令注册只通过下面的 meta_manager 单一入口）
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

/// 统一认证 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `meta_manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn meta_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    meta_manager::dispatch(state, app, req).await
}
