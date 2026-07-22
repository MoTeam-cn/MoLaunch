//! 账号管理命令（列表/删除/切换/登出/状态恢复）
//!
//! 模块结构：
//! - mod.rs: 数据类型（MsAccountInfo / OfflineAccountInfo）+ 模块入口
//! - ms.rs: 微软账号管理命令（get_ms_accounts / remove_ms_account / switch_ms_account）
//! - offline.rs: 离线账号管理命令（get_offline_accounts / set_offline_skin /
//!   save_custom_skin / remove_offline_account / switch_offline_account）
//! - session.rs: 会话命令（get_login_status / logout）
//!
//! 注意：所有 #[tauri::command] 命令分散到 ms/offline/session 子模块，
//! tauri::command 宏在定义处生成 __cmd__ 符号，不能通过 pub use 重导出，
//! 故 lib.rs 使用完整路径注册（commands::auth::account::ms::* / ::offline::* / ::session::*）

pub mod ms;
pub mod offline;
pub mod session;

use serde::Serialize;

/// 已存储的微软账号信息
#[derive(Debug, Clone, Serialize)]
pub struct MsAccountInfo {
    pub username: String,
    pub uuid: String,
    pub expires_at: u64,
    pub is_expired: bool,
}

/// 已存储的离线账号信息
#[derive(Debug, Clone, Serialize)]
pub struct OfflineAccountInfo {
    pub username: String,
    pub uuid: String,
    pub skin: Option<String>,
}
