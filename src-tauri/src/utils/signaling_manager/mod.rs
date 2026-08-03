//! 信令 action 管理器（房间创建/加入/退出/踢人/保活等）
//!
//! 由 `online_manager::DISPATCHER` 调用 `register_signaling_actions` 注册。
//! 子模块：params（参数结构体）/ registry（注册入口 + 凭证/客户端辅助）/
//! lobby_actions / room_actions / session_actions / whitelist_actions

mod lobby_actions;
mod params;
mod registry;
mod room_actions;
mod session_actions;
mod whitelist_actions;

use registry::{load_creds, make_client};

pub use params::*;
pub use registry::register_signaling_actions;