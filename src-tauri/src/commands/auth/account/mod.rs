//! 账号管理命令（列表/删除/切换/登出/状态恢复）
//! 子模块：ms（微软账号 get/remove/switch）/ offline（离线账号 get/set_skin/save_custom_skin/
//! remove/switch）/ session（get_login_status/logout）。函数去 `#[tauri::command]` 标注改收
//! `&AppState`，由 `commands::auth::meta_manager` 统一 IPC 入口通过 dispatch 分发调用。

pub mod info;
pub mod ms;
pub mod offline;
pub mod session;

pub use info::{MsAccountInfo, OfflineAccountInfo};
