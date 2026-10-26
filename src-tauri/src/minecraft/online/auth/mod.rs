//! MoSign-v1 设备认证（转发到私有协议库）
//!
//! 注册 / 登录 / 刷新的请求构造、响应解析与协议类型都在 `molaunch-protocol::auth`。
//! 持久化、HTTP 发送与状态机留在本仓库。

pub use molaunch_protocol::auth::*;
