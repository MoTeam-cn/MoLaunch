//! 实验性功能 action 分发（模块化）
//!
//! 子模块：`dispatcher`（注册表）/ `chat`（聊天动作）/ `context`（上下文构建）/
//! `tool_loop`（工具循环）/ `analyze`（AI 日志分析）/ `emit`（流式事件与标题）/
//! `common`（开关校验与上下文构建公共辅助）。

mod analyze;
mod chat;
mod common;
mod compression;
mod context;
mod dispatcher;
mod emit;
mod tool_loop;

pub use dispatcher::dispatch;
