//! 信令接口的请求/响应类型定义（ICE、整合包、房间核心类型）
//!
//! 对应 api-server `/v1/signaling/*` 的请求体与响应体结构。
//! 拆分为子模块：`ice`（配置兼容占位）、`room`（整合包+房间）。

mod ice;
mod room;

pub use ice::*;
pub use room::*;
