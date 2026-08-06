//! AI Agent：工具定义、执行与上下文收集
//!
//! 子模块：`tools`（AgentContext + 工具定义/执行）/ `logs` / `crash` / `info`（只读诊断数据）/
//! `ask`（ask_user 提问与回填）。

mod ask;
mod crash;
mod info;
mod logs;
mod tools;

pub use ask::reply_ask_user;
pub use tools::{collect_context, execute_tool, tool_definitions, AgentContext};
