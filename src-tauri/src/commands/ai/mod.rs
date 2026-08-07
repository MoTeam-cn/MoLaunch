//! AI 纯实现模块，无独立 IPC 入口；服务层位于 `crate::ai_core`。
//! AI action 由实验性命令统一分发，本模块提供可复用的实现与类型。

pub mod manager;
pub mod types;
