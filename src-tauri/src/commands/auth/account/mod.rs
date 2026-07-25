//! 账号管理命令（列表/删除/切换/登出/状态恢复）
//!
//! 模块结构：
//! - mod.rs: 数据类型（MsAccountInfo / OfflineAccountInfo）+ 模块入口
//! - ms.rs: 微软账号管理命令（get_ms_accounts / remove_ms_account / switch_ms_account）
//! - offline.rs: 离线账号管理命令（get_offline_accounts / set_offline_skin /
//!   save_custom_skin / remove_offline_account / switch_offline_account）
//! - session.rs: 会话命令（get_login_status / logout）
//!
//! 注：原 `#[tauri::command]` 标注已移除，所有函数改为接收 `&AppState`，
//! 由 `commands::auth::meta_manager` 统一 IPC 入口通过
//! `utils::meta_manager::dispatch` 分发调用。

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
