/**
 * 实验性功能模块 API（聊天记录 SQLite 存储 / Agent 对话 / 日志分析）
 *
 * 对应后端 `experimental_manager` IPC 命令，通过 `action` 字段分发。
 * 所有操作需先开启「设置 → 进阶设置 → 实验性功能」开关，否则后端返回错误。
 *
 * 聊天为流式：`chat_send` / `regenerate_reply` / `edit_message` 返回后，
 * 增量通过 `ai-chat-stream` 事件逐字推送（见 `AiChatStreamEvent`）。
 */

import { invoke } from '@tauri-apps/api/core'

/** 调用 experimental_manager IPC */
export async function experimentalManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('experimental_manager', { req: { action, params: params ?? null } })
}

/** 所有可用的 action 名称（与后端 commands::experimental::manager 注册一致） */
export const EXPERIMENTAL_ACTIONS = {
  CREATE_CONVERSATION: 'create_conversation',
  LIST_CONVERSATIONS: 'list_conversations',
  DELETE_CONVERSATION: 'delete_conversation',
  RENAME_CONVERSATION: 'rename_conversation',
  LIST_MESSAGES: 'list_messages',
  LIST_TOOL_CALLS: 'list_tool_calls',
  CLEAR_CONVERSATION: 'clear_conversation',
  CHAT_SEND: 'chat_send',
  COLLECT_CONTEXT: 'collect_context',
  DELETE_MESSAGE: 'delete_message',
  REGENERATE_REPLY: 'regenerate_reply',
  EDIT_MESSAGE: 'edit_message',
  REPLY_ASK_USER: 'reply_ask_user',
  CANCEL_CHAT: 'cancel_chat',
  LIST_INSTALLED_VERSIONS: 'list_installed_versions',
  AI_ANALYZE_LOG: 'ai_analyze_log',
} as const

/** action 名称类型 */
export type ExperimentalAction = typeof EXPERIMENTAL_ACTIONS[keyof typeof EXPERIMENTAL_ACTIONS]

/** 会话项（与后端 ConversationItem 对应，字段为 camelCase） */
export interface ConversationItem {
  id: number
  title: string
  createdAt: number
  updatedAt: number
}

/** 消息项（与后端 MessageItem 对应，字段为 camelCase） */
export interface MessageItem {
  id: number
  role: string
  content: string
  createdAt: number
  /** 配对消息 id（用户↔AI 一一配对，删除时级联） */
  pairId: number | null
  /** 该消息对应的游戏版本（AI 工具调用时记录） */
  versionId: string | null
  /** 思考模型的推理内容（展示为可折叠「深度思考」区块） */
  reasoningContent?: string | null
  /** 生成该回复的模型名（AI 消息固定展示，切换全局模型不影响历史消息） */
  model?: string | null
  /** 该回复的生成序号（首次为 1，重新生成递增，用于「第 N 次重试」标识） */
  retryCount?: number | null
  /** 本次回复（含全部工具调用轮次）消耗的输入 token（后端流式 usage 累计） */
  promptTokens?: number | null
  /** 本次回复（含全部工具调用轮次）生成的输出 token */
  completionTokens?: number | null
  /** 本次回复总 token（prompt + completion） */
  totalTokens?: number | null
  /** 本次回复连贯生成耗时（ms，排除 ask_user 等待人类回答的时间） */
  durationMs?: number | null
}

/** 工具调用记录（与后端 ToolCallRecord 对应，随 AI 回复消息持久化到 SQLite） */
export interface ToolCallRecord {
  /** 所属 AI 回复消息 id */
  messageId: number
  /** 工具在调用链中的顺序 */
  seq: number
  /** 工具名 */
  name: string
  /** 工具入参（JSON 字符串） */
  arguments: string
  /** 工具执行输出（失败时为错误说明文本） */
  output: string | null
  /** 调用该工具前模型输出的过渡文本（同一轮内多个工具共享） */
  preContent?: string | null
}

/** 发送聊天消息参数（字段为 camelCase，与后端 ChatSendParams 对应） */
export interface ChatSendParams {
  conversationId: number
  content: string
  /** 手动附加上下文（模型不支持工具调用时的兜底） */
  attachContext?: string
  /** 本次对话覆盖的模型名（留空使用默认模型） */
  model?: string
  /** 手动附加上下文时对应的游戏版本 */
  versionId?: string
  /** 思考程度（low/medium/high；关闭思考模式时传 null，后端透传 reasoning_effort） */
  reasoningEffort?: string | null
}

/** 聊天发送结果 */
export interface ChatSendResult {
  conversationId: number
  /** 最终生成的 AI 回复（流式增量由事件推送） */
  reply: string
  /** 本次对话实际触发的工具调用记录（供前端展示 Agent 行为） */
  toolCallsLog: string[]
}

/** 删除消息结果 */
export interface DeleteMessageResult {
  /** 被删除的消息 id（含配对消息） */
  deletedIds: number[]
}

/** 收集上下文结果 */
export interface CollectContextResult {
  kind: string
  text: string
}

/** 收集上下文参数 */
export interface CollectContextParams {
  /** launcher | game_logs | crash_report | mods | launcher_logs */
  kind: string
  /** 游戏版本 id（版本隔离下必须提供） */
  versionId?: string
  /** 会话 id（可选） */
  conversationId?: number
}

/** 创建会话 */
export function experimentalCreateConversation(title?: string): Promise<ConversationItem> {
  return experimentalManager<ConversationItem>(EXPERIMENTAL_ACTIONS.CREATE_CONVERSATION, {
    title: title ?? '',
  })
}

/** 会话列表（按最近更新倒序） */
export function experimentalListConversations(): Promise<ConversationItem[]> {
  return experimentalManager<ConversationItem[]>(EXPERIMENTAL_ACTIONS.LIST_CONVERSATIONS)
}

/** 删除会话（级联删除消息） */
export function experimentalDeleteConversation(conversationId: number): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.DELETE_CONVERSATION, {
    conversationId,
  })
}

/** 重命名会话 */
export function experimentalRenameConversation(conversationId: number, title: string): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.RENAME_CONVERSATION, {
    conversationId,
    title,
  })
}

/** 读取会话消息（按时间正序） */
export function experimentalListMessages(conversationId: number): Promise<MessageItem[]> {
  return experimentalManager<MessageItem[]>(EXPERIMENTAL_ACTIONS.LIST_MESSAGES, {
    conversationId,
  })
}

/** 读取会话内全部工具调用记录（前端按 message_id 分组展示各消息的工具链） */
export function experimentalListToolCalls(conversationId: number): Promise<ToolCallRecord[]> {
  return experimentalManager<ToolCallRecord[]>(EXPERIMENTAL_ACTIONS.LIST_TOOL_CALLS, {
    conversationId,
  })
}

/** 清空会话消息（保留会话） */
export function experimentalClearConversation(conversationId: number): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.CLEAR_CONVERSATION, {
    conversationId,
  })
}

/** 发送聊天消息（流式：增量通过 ai-chat-stream 事件推送） */
export function experimentalChatSend(params: ChatSendParams): Promise<ChatSendResult> {
  return experimentalManager<ChatSendResult>(EXPERIMENTAL_ACTIONS.CHAT_SEND, params)
}

/** 删除消息（级联配对：删除 AI 消息同时删除对应用户消息，反之亦然） */
export function experimentalDeleteMessage(
  conversationId: number,
  messageId: number,
): Promise<DeleteMessageResult> {
  return experimentalManager<DeleteMessageResult>(EXPERIMENTAL_ACTIONS.DELETE_MESSAGE, {
    conversationId,
    messageId,
  })
}

/** 重新回复（对某条 AI 消息，用其对应的用户消息重新生成；流式） */
export function experimentalRegenerateReply(
  conversationId: number,
  messageId: number,
  model?: string,
  reasoningEffort?: string | null,
): Promise<ChatSendResult> {
  return experimentalManager<ChatSendResult>(EXPERIMENTAL_ACTIONS.REGENERATE_REPLY, {
    conversationId,
    messageId,
    model,
    reasoningEffort,
  })
}

/** 编辑消息（仅最近一条用户消息可编辑；编辑后自动重新生成回复；流式） */
export function experimentalEditMessage(
  conversationId: number,
  messageId: number,
  content: string,
  model?: string,
  reasoningEffort?: string | null,
): Promise<ChatSendResult> {
  return experimentalManager<ChatSendResult>(EXPERIMENTAL_ACTIONS.EDIT_MESSAGE, {
    conversationId,
    messageId,
    content,
    model,
    reasoningEffort,
  })
}

/** 回填 ask_user 提问结果 */
export function experimentalReplyAskUser(
  conversationId: number,
  reply: string,
): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.REPLY_ASK_USER, {
    conversationId,
    reply,
  })
}

/** 取消当前正在进行的流式回复（模型回复期间点击暂停按钮调用） */
export function experimentalCancelChat(): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.CANCEL_CHAT)
}

/** 已安装游戏版本列表 */
export function experimentalListInstalledVersions(): Promise<string[]> {
  return experimentalManager<string[]>(EXPERIMENTAL_ACTIONS.LIST_INSTALLED_VERSIONS)
}

/** 收集上下文（手动附加上下文兜底：launcher / game_logs / crash_report / mods / launcher_logs） */
export function experimentalCollectContext(params: CollectContextParams): Promise<CollectContextResult> {
  return experimentalManager<CollectContextResult>(EXPERIMENTAL_ACTIONS.COLLECT_CONTEXT, params)
}

/** ai-chat-stream 事件的 usage 负载 */
export interface AiChatUsage {
  promptTokens: number
  completionTokens: number
  totalTokens: number
}

/** ai-chat-stream 事件负载（后端 emit） */
export interface AiChatStreamEvent {
  conversationId: number
  /** 内容增量（逐字推送） */
  delta?: string
  /** 思考内容增量（思考模型，如 DeepSeek-R1，通过 delta.reasoning_content 推送） */
  reasoning?: string
  /** 该轮对话完成（含 usage 校准） */
  done?: boolean
  /** 工具调用状态（index 关联 running/done） */
  toolCall?: {
    name: string
    status: 'running' | 'done'
    /** 全局唯一序号（如 r0-0），用于关联开始与完成 */
    index?: string
    /** 工具入参 JSON（running 时携带） */
    arguments?: string
    /** 调用该工具前模型输出的过渡文本（running 时携带） */
    preContent?: string
    /** 工具执行结果（done 时携带） */
    output?: string
  }
  /** 上下文压缩等状态提示 */
  status?: string
  /** 最终 usage（校准 token 用量） */
  usage?: AiChatUsage
  /** 本次回复连贯生成耗时（ms，后端已扣除 ask_user 等待人类回答的时间） */
  durationMs?: number
}

/** ai-ask-user 事件的选项项：label 为选项文本，description 为选项备注/注释 */
export interface AskUserOption {
  label: string
  description?: string
}

/** ai-ask-user 事件负载（模型向用户提问） */
export interface AiAskUserEvent {
  conversationId: number
  question: string
  options: AskUserOption[]
}

/** AI 日志分析参数（ai_analyze_log） */
export interface AiAnalyzeLogParams {
  logText: string
  model: string
  /** 思考程度（low/medium/high；关闭时传 null） */
  reasoningEffort?: string | null
  /** 本地预检：true 时后端先用本地规则引擎收敛范围再注入 AI 分析（避免超长全文直发模型） */
  localAnalyze?: boolean
}

/** ai-analyze-stream 事件负载（AI 日志分析页专用，独立于 ai-chat-stream） */
export interface AiAnalyzeStreamEvent {
  /** 结论内容增量（流式） */
  delta?: string
  /** 环节进度（1~5，来自模型输出的【STEP:N/5】标记） */
  step?: number
  /** 分析完成（携带剔除标记后的全文） */
  done?: boolean
  content?: string
}

/** AI 日志分析（流式：增量与环节进度通过 ai-analyze-stream 事件推送） */
export function experimentalAiAnalyzeLog(params: AiAnalyzeLogParams): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.AI_ANALYZE_LOG, params)
}
