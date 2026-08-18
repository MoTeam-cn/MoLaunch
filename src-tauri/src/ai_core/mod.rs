//! AI 服务核心（OpenAI 兼容 API 客户端 + 提示词构造 + 配置持久化）
//!
//! 服务层，不含 Tauri 依赖；被 `commands/ai` IPC 层调用。
//! 服务地址为本地 OpenAI 兼容 API（如 Ollama / LM Studio），不依赖云端。

pub mod client;
pub mod config;
pub mod prompt;
pub mod storage;

pub use client::{
    chat, chat_completions, chat_completions_stream, chat_json, estimate_tokens, list_models,
    ChatResult, ChatTurn, StreamCallbacks, StreamToolDelta, StreamUsage, ToolCall, ToolDef,
};
pub use config::AiConfig;
pub use prompt::PromptKind;
pub use storage::{load as load_config, load_async as load_config_async, save as save_config};
