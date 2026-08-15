//! 信令 action 管理器（Scaffolding 收敛版：房间创建/查询/加入/关闭 + 大厅浏览）
//!
//! 由 `online_manager::DISPATCHER` 调用 `register_signaling_actions` 注册。
//! 子模块：params（参数结构体）/ registry（注册入口 + 凭证/客户端辅助）/
//! lobby_actions / room_actions

mod lobby_actions;
mod params;
mod registry;
mod room_actions;

use registry::{load_creds, make_client};

pub use params::*;
pub use registry::register_signaling_actions;
