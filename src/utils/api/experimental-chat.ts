import { experimentalManager } from './experimental'
import { EXPERIMENTAL_ACTIONS } from './experimental-actions'

export interface ChatSendParams {
  conversationId: number
  content: string
  attachContext?: string
  model?: string
  versionId?: string
  reasoningEffort?: string | null
}

export interface ChatSendResult {
  conversationId: number
  reply: string
  toolCallsLog: string[]
}

export interface DeleteMessageResult {
  deletedIds: number[]
}

export function experimentalChatSend(params: ChatSendParams): Promise<ChatSendResult> {
  return experimentalManager<ChatSendResult>(EXPERIMENTAL_ACTIONS.CHAT_SEND, params)
}

export function experimentalDeleteMessage(conversationId: number, messageId: number): Promise<DeleteMessageResult> {
  return experimentalManager<DeleteMessageResult>(EXPERIMENTAL_ACTIONS.DELETE_MESSAGE, { conversationId, messageId })
}

export function experimentalRegenerateReply(
  conversationId: number,
  messageId: number,
  model?: string,
  reasoningEffort?: string | null,
): Promise<ChatSendResult> {
  return experimentalManager<ChatSendResult>(EXPERIMENTAL_ACTIONS.REGENERATE_REPLY, {
    conversationId, messageId, model, reasoningEffort,
  })
}

export function experimentalEditMessage(
  conversationId: number,
  messageId: number,
  content: string,
  model?: string,
  reasoningEffort?: string | null,
): Promise<ChatSendResult> {
  return experimentalManager<ChatSendResult>(EXPERIMENTAL_ACTIONS.EDIT_MESSAGE, {
    conversationId, messageId, content, model, reasoningEffort,
  })
}

export function experimentalReplyAskUser(conversationId: number, reply: string): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.REPLY_ASK_USER, { conversationId, reply })
}

export function experimentalCancelChat(): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.CANCEL_CHAT)
}
