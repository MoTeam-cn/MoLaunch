//! 信令接口的请求/响应类型定义（ICE、整合包、房间核心类型）
//!
//! 对应 api-server `/v1/signaling/*` 的请求体与响应体结构。
//! 拆分为子模块：`ice`（ICE/STUN/TURN）、`room`（整合包+房间）、`session`（参与会话）。

mod ice;
mod room;
mod session;

pub use ice::*;
pub use room::*;
pub use session::*;
