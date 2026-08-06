//! AI 客户端类型定义（请求/响应结构、工具调用、流式回调）

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 单轮聊天（无工具，旧接口 chat()）
// ---------------------------------------------------------------------------

/// 聊天请求体（单轮）
#[derive(Debug, Serialize)]
pub(crate) struct ChatRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChatMessage {
    pub(crate) role: &'static str,
    pub(crate) content: String,
}

/// 聊天响应（单轮）
#[derive(Debug, Deserialize)]
pub(crate) struct ChatResponse {
    pub(crate) choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatChoice {
    pub(crate) message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatResponseMessage {
    pub(crate) content: Option<String>,
}

// ---------------------------------------------------------------------------
// 模型列表
// ---------------------------------------------------------------------------

/// 模型列表响应（OpenAI 兼容 `GET /models`）
#[derive(Debug, Deserialize)]
pub(crate) struct ModelsResponse {
    pub(crate) data: Vec<ModelsData>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ModelsData {
    pub(crate) id: Option<String>,
}

// ---------------------------------------------------------------------------
// 多轮 + 工具调用
// ---------------------------------------------------------------------------

/// 聊天消息（多轮 / 工具调用场景下的通用消息体）
///
/// - 普通消息：`role` + `content`
/// - 模型发起工具调用：`role = "assistant"` + `tool_calls`
/// - 工具执行结果：`role = "tool"` + `tool_call_id` + `name` + `content`
#[derive(Debug, Clone, Serialize)]
pub struct ChatTurn {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// 思考模型（如 DeepSeek-R1）的推理内容；涉及工具调用的轮次必须完整回传，否则服务返回 400
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 工具定义（随请求下发，模型可据此发起 tool_calls）
#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    #[serde(rename = "type")]
    pub ty: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 模型发起的工具调用（响应侧）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type", default = "default_tool_call_type")]
    pub ty: String,
    pub function: ToolCallFunction,
}

fn default_tool_call_type() -> String {
    "function".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

/// 聊天完成结果
#[derive(Debug, Clone, Default)]
pub struct ChatResult {
    pub content: Option<String>,
    /// 思考模型的推理内容（流式时逐块累计）
    pub reasoning_content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

/// 多轮 + 工具调用聊天请求体（OpenAI 兼容）
#[derive(Debug, Serialize)]
pub(crate) struct ChatCompletionsRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<ChatTurn>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tools: Option<Vec<ToolDef>>,
    pub(crate) stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_tokens: Option<u32>,
    /// 思考程度：low | medium | high（透传式，服务端不支持时自动忽略）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning_effort: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatCompletionsResponse {
    pub(crate) choices: Vec<ChatCompletionsChoice>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatCompletionsChoice {
    pub(crate) message: ChatCompletionsMessage,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatCompletionsMessage {
    pub(crate) content: Option<String>,
    #[serde(default)]
    pub(crate) reasoning_content: Option<String>,
    #[serde(default)]
    pub(crate) tool_calls: Vec<ToolCall>,
}

// ---------------------------------------------------------------------------
// 流式回调
// ---------------------------------------------------------------------------

/// 流式增量回调：每个 SSE 块到达时立即调用
///
/// - `delta`：本轮增量文本（可为空，如工具调用块）
/// - `reasoning_delta`：思考模型的推理内容增量（`delta.reasoning_content`，可为空）
/// - `tool_calls`：增量工具调用（按 index 聚合，由调用方累计）
/// - `done`：该轮完成（`finish_reason == "stop"` 或流结束）
/// - `usage`：最终 usage（部分服务在流末返回）
pub struct StreamCallbacks {
    pub on_delta: Box<dyn Fn(&str) + Send + Sync>,
    pub on_reasoning_delta: Box<dyn Fn(&str) + Send + Sync>,
    pub on_tool_delta: Box<dyn Fn(&StreamToolDelta) + Send + Sync>,
    pub on_done: Box<dyn Fn(&StreamUsage) + Send + Sync>,
}

/// 流式工具调用增量
#[derive(Debug, Clone, Default)]
pub struct StreamToolDelta {
    pub index: usize,
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments: String,
}

/// 流式最终 usage
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}
