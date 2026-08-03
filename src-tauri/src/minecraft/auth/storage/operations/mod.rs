//! AuthStorage 高层操作（独立 impl 块）
//!
//! 按业务域拆分：ms（微软/离线账号）、authlib（外置登录账号）。

mod authlib;
mod ms;