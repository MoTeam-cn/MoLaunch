//! P2P 联机信令接口客户端
//!
//! 对接 api-server `/v1/signaling/*` 接口，实现房间创建/加入/退出/踢人/保活等。
//! 接口参考：`api-server/docs/signaling.md`
//!
//! 拆分为子模块：`types`（类型定义）、`room_api`（房间生命周期）、
//! `session`（会话/封禁/Offer）、`whitelist`（白名单）、`lobby`（大厅浏览）。

mod lobby;
mod room_api;
mod session;
mod types;
mod whitelist;

pub use lobby::*;
pub use session::*;
pub use types::*;
pub use whitelist::*;
