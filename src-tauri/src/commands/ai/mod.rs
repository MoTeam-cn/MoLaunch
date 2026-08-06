//! AI 模块（实现库，无独立 IPC 入口）
//! 子模块：manager（分析/状态查询实现）、types（IPC 类型）。
//! 服务层逻辑位于 `crate::ai_core`（不含 Tauri 依赖），
//! 服务为本地 OpenAI 兼容 API（如 Ollama / LM Studio）。
//!
//! 自「实验性」功能上线后，所有 AI action 已并入
//! `commands::experimental::experimental_manager` 统一分发（见
//! `commands/experimental/manager.rs` 的 ai action 注册），
//! 本模块仅提供被复用的纯实现函数，不再注册独立 Tauri 命令。

pub mod manager;
pub mod types;
