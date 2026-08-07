import type { ComputedRef, Ref } from 'vue'
import type { ConversationItem, AiChatStreamEvent, AiAskUserEvent, AskUserOption } from '@/utils/api/experimental'
import type { LocalMessage } from '@/components/experimental/ChatMessageItem.vue'

export interface ToolCallItem {
  index: string
  name: string
  status: 'running' | 'done'
  arguments: string
  output: string
  preContent: string
}

export interface AiChatState {
  conversations: Ref<ConversationItem[]>
  activeId: Ref<number>
  activeTitle: Ref<string>
  messages: Ref<LocalMessage[]>
  inputText: Ref<string>
  loading: Ref<boolean>
  creating: Ref<boolean>
  currentModel: Ref<string>
  enableReasoning: Ref<boolean>
  reasoningLevel: Ref<'low' | 'medium' | 'high'>
  scrolledUp: Ref<boolean>
  lastUsage: Ref<AiChatStreamEvent['usage'] | null>
  chatSpeed: Ref<number>
  toolCalls: Ref<ToolCallItem[]>
  toolCallsByMessage: Ref<Record<number, ToolCallItem[]>>
  streamingMsg: Ref<LocalMessage | null>
  askUser: Ref<{ visible: boolean; question: string; options: AskUserOption[] }>
  versionPicker: Ref<{ visible: boolean; versions: string[] }>
  pendingAttachKind: Ref<string>
  waitingAsk: ComputedRef<boolean>
}

export type ChatStreamEvent = AiChatStreamEvent
export type ChatAskUserEvent = AiAskUserEvent
