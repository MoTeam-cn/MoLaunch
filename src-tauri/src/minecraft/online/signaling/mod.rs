//! P2P 联机信令接口客户端（Scaffolding 收敛版）
//! 对接 api-server `/v1/signaling/*` 接口：房间创建/查询/加入/关闭、大厅聚合与公开房间列表。
//! 拆分为子模块：`types`（类型定义）、`room_api`（房间生命周期）、`lobby`（大厅浏览）。

mod lobby;
mod room_api;
mod types;

pub use lobby::*;
pub use types::*;
