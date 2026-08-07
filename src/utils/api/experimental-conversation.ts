import { experimentalManager } from './experimental'
import { EXPERIMENTAL_ACTIONS } from './experimental-actions'

export interface ConversationItem {
  id: number
  title: string
  createdAt: number
  updatedAt: number
}

export interface MessageItem {
  id: number
  role: string
  content: string
  createdAt: number
  pairId: number | null
  versionId: string | null
  reasoningContent?: string | null
  model?: string | null
  retryCount?: number | null
  promptTokens?: number | null
  completionTokens?: number | null
  totalTokens?: number | null
  durationMs?: number | null
}

export interface ToolCallRecord {
  messageId: number
  seq: number
  name: string
  arguments: string
  output: string | null
  preContent?: string | null
}

export function experimentalCreateConversation(title?: string): Promise<ConversationItem> {
  return experimentalManager<ConversationItem>(EXPERIMENTAL_ACTIONS.CREATE_CONVERSATION, { title: title ?? '' })
}

export function experimentalListConversations(): Promise<ConversationItem[]> {
  return experimentalManager<ConversationItem[]>(EXPERIMENTAL_ACTIONS.LIST_CONVERSATIONS)
}

export function experimentalDeleteConversation(conversationId: number): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.DELETE_CONVERSATION, { conversationId })
}

export function experimentalRenameConversation(conversationId: number, title: string): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.RENAME_CONVERSATION, { conversationId, title })
}

export function experimentalListMessages(conversationId: number): Promise<MessageItem[]> {
  return experimentalManager<MessageItem[]>(EXPERIMENTAL_ACTIONS.LIST_MESSAGES, { conversationId })
}

export function experimentalListToolCalls(conversationId: number): Promise<ToolCallRecord[]> {
  return experimentalManager<ToolCallRecord[]>(EXPERIMENTAL_ACTIONS.LIST_TOOL_CALLS, { conversationId })
}

export function experimentalClearConversation(conversationId: number): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.CLEAR_CONVERSATION, { conversationId })
}
