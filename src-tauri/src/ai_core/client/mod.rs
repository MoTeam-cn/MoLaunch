//! AI 客户端模块：提供非流式、流式请求、Token 估算及请求类型定义。

mod accumulator;
pub(crate) mod chat;
mod sse;
mod stream;
mod tokens;
mod transport;
mod types;

pub use chat::{chat, chat_completions, list_models};
pub use stream::chat_completions_stream;
pub use tokens::estimate_tokens;
pub use types::{
    ChatResult, ChatTurn, StreamCallbacks, StreamToolDelta, StreamUsage, ToolCall, ToolDef,
    ToolFunction,
};
