//! 深度链接（Deep Link）模块：注册 `molaunch://` 协议并提供注册式路由分发入口

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
