//! 深度链接（Deep Link）模块入口
//!
//! 注册 `molaunch://` 协议，并采用注册式分发模式：任意模块可调用
//! [`register`] 注册后缀路由（如 `molaunch://run`），URL 到达时按 host 段分发。
//!
//! 本文件**仅作入口**：模块声明 + 公共 API re-export。业务逻辑拆分到：
//! - [`request`]：URL 解析与 [`DeeplinkRequest`] 结构
//! - [`router`]：路由注册表 / 分发 / 初始化
//! - [`handlers`]：内置 handler（run / install / open）
//! - [`security`]：下载域名白名单安全校验
//! - [`protocol`]：协议注册/卸载/状态（便携版无安装器时的方案）
//!
//! # 支持的路由（见 handlers）
//! - `molaunch://run` —— 启动游戏
//! - `molaunch://install?url=xxx` —— 安装整合包（强制白名单校验）
//! - `molaunch://open?page=xxx` —— 打开前端指定页面
//!
//! # 扩展方式
//! 业务模块直接 `deeplink::register("xxx", handler)` 注册新路由，无需改本模块。

mod handlers;
mod protocol;
mod request;
mod router;
mod security;

pub use protocol::{
    auto_register, register as register_protocol, status as protocol_status,
    unregister as unregister_protocol, DeeplinkStatus,
};
pub use request::{parse, DeeplinkRequest};
pub use router::{dispatch, init, register, register_sync};
