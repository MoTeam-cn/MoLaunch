//! AI 客户端实现（模块化拆分，避免单文件堆积）
//!
//! - `types`：请求/响应结构、工具调用、流式回调类型
//! - `transport`：HTTP 传输层（复用 `crate::http` 全局客户端；代理/IP/TLS 变更后
//!   由 `apply_config` 重建，无需重启即热生效；AI 配置每次调用由上层
//!   `load_config_async()` 重新读取，同样支持热重载）
//! - `tokens`：本地 token 估算（与前端 `src/utils/tokens.ts` 对齐）
//! - `chat`：非流式接口（`chat` / `list_models` / `chat_completions`）
//! - `stream`：流式接口（`chat_completions_stream`，SSE 逐块解析）

pub(crate) mod chat;
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
