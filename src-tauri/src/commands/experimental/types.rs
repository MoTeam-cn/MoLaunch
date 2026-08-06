//! 实验性功能 IPC 类型定义

use serde::{Deserialize, Serialize};

/// 会话项（聊天会话列表）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationItem {
    pub id: i64,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 消息项
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageItem {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at: i64,
    /// 配对消息 id（用户↔AI 一一配对，删除时级联）
    pub pair_id: Option<i64>,
    /// 该消息对应的游戏版本（AI 工具调用时记录）
    pub version_id: Option<String>,
    /// 思考模型的推理内容（展示为可折叠「深度思考」区块）
    pub reasoning_content: Option<String>,
    /// 生成该回复的模型名（AI 消息固定展示其回复模型，切换全局模型不影响历史消息）
    pub model: Option<String>,
    /// 该回复的生成序号（首次为 1，重新生成递增，用于「第 N 次重试」标识）
    pub retry_count: Option<i64>,
    /// 本次回复（含全部工具调用轮次）消耗的输入 token
    pub prompt_tokens: Option<i64>,
    /// 本次回复（含全部工具调用轮次）生成的输出 token
    pub completion_tokens: Option<i64>,
    /// 本次回复总 token（prompt + completion）
    pub total_tokens: Option<i64>,
    /// 本次回复连贯生成耗时（ms，排除 ask_user 等待人类回答的时间）
    pub duration_ms: Option<i64>,
}

/// 工具调用记录（入库后随 AI 回复消息持久化，前端按消息展示工具链）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallRecord {
    /// 所属 AI 回复消息 id
    pub message_id: i64,
    /// 工具在调用链中的顺序（从 0 起，跨轮累计）
    pub seq: i64,
    /// 工具名
    pub name: String,
    /// 工具入参（JSON 字符串）
    pub arguments: String,
    /// 工具执行输出（失败时为错误说明文本）
    pub output: Option<String>,
    /// 调用该工具前模型输出的过渡文本（同一轮内多个工具共享）
    pub pre_content: Option<String>,
}

/// 创建会话入参
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationParams {
    #[serde(default)]
    pub title: Option<String>,
}

/// 删除会话入参
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationIdParams {
    pub conversation_id: i64,
}

/// 重命名会话入参
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameConversationParams {
    pub conversation_id: i64,
    pub title: String,
}

/// 读取消息入参
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMessagesParams {
    pub conversation_id: i64,
}

/// 读取工具调用记录入参
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListToolCallsParams {
    pub conversation_id: i64,
}

/// 发送聊天消息入参
///
/// - `attach_context`：手动附加上下文兜底（模型不支持工具调用时使用）
/// - `model`：本次对话覆盖的模型名（留空使用默认模型）
/// - `version_id`：手动附加上下文时对应的游戏版本
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSendParams {
    pub conversation_id: i64,
    pub content: String,
    #[serde(default)]
    pub attach_context: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
    /// 思考程度：low | medium | high（透传式，服务端不支持时自动忽略）
    #[serde(default)]
    pub reasoning_effort: Option<String>,
}

/// 删除消息入参（级联配对：删除 AI 消息会同时删除对应用户消息，反之亦然）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMessageParams {
    pub conversation_id: i64,
    pub message_id: i64,
}

/// 重新回复入参（对某条 AI 消息，使用其对应的用户消息重新生成）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateReplyParams {
    pub conversation_id: i64,
    pub message_id: i64,
    #[serde(default)]
    pub model: Option<String>,
    /// 思考程度：low | medium | high（透传式，服务端不支持时自动忽略）
    #[serde(default)]
    pub reasoning_effort: Option<String>,
}

/// 编辑消息入参（仅最近一条用户消息可编辑；编辑后删除其后 AI 回复并重新生成）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditMessageParams {
    pub conversation_id: i64,
    pub message_id: i64,
    pub content: String,
    #[serde(default)]
    pub model: Option<String>,
    /// 思考程度：low | medium | high（透传式，服务端不支持时自动忽略）
    #[serde(default)]
    pub reasoning_effort: Option<String>,
}

/// AI 日志分析入参（5 环节流式）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAnalyzeLogParams {
    /// 日志全文（必填）
    pub log_text: String,
    /// 可选显式指定模型；为空时使用默认模型
    #[serde(default)]
    pub model: Option<String>,
    /// 思考程度：low | medium | high（透传式，服务端不支持时自动忽略）
    #[serde(default)]
    pub reasoning_effort: Option<String>,
    /// 本地预检：true 时先用本地规则引擎对日志初检收敛范围，把「本地初检结果摘要」作为
    /// 上下文注入 AI 分析（避免把超长全文直接发给模型）；false 时直接发送原文
    #[serde(default)]
    pub local_analyze: bool,
}

/// 回填 ask_user 提问结果入参
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyAskUserParams {
    pub conversation_id: i64,
    pub reply: String,
}

/// 聊天发送结果（流式：前端通过事件接收增量，此结构仅承载会话 id 与工具记录）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSendResult {
    pub conversation_id: i64,
    pub reply: String,
    /// 本次对话中实际触发的工具调用记录（供前端展示 Agent 行为）
    #[serde(default)]
    pub tool_calls_log: Vec<String>,
}

/// 收集上下文入参（手动附加上下文，模型不支持工具时的兜底）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectContextParams {
    /// launcher | game_logs | crash_report | mods | launcher_logs
    pub kind: String,
    /// 游戏版本 id（版本隔离下必须提供，用于定位该版本的数据目录）
    pub version_id: Option<String>,
    /// 会话 id（可选，构造 AgentContext 用）
    #[serde(default)]
    pub conversation_id: Option<i64>,
}

/// 收集上下文结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectContextResult {
    pub kind: String,
    pub text: String,
}
