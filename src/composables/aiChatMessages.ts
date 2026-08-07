import { ref, computed, type Ref } from 'vue'
import { experimentalListMessages, experimentalListToolCalls, type ConversationItem } from '@/utils/api/experimental'
import { safeCall } from '@/utils/async'
import type { LocalMessage } from '@/components/experimental/ChatMessageItem.vue'
import type { ToolCallItem } from './aiChatTypes'

export function useAiChatMessages(activeId: Ref<number>) {
  const messages = ref<LocalMessage[]>([])
  const toolCallsByMessage = ref<Record<number, ToolCallItem[]>>({})

  async function refreshMessages() {
    if (!activeId.value) return
    const [items, records] = await Promise.all([
      safeCall(() => experimentalListMessages(activeId.value), 'list messages', () => null),
      safeCall(() => experimentalListToolCalls(activeId.value), 'list tool calls', () => null),
    ])
    if (items) messages.value = items
    const grouped: Record<number, ToolCallItem[]> = {}
    for (const r of records ?? []) {
      const list = (grouped[r.messageId] ??= [])
      list.push({
        index: `db-${r.seq}`,
        name: r.name,
        status: 'done',
        arguments: r.arguments,
        output: r.output ?? '',
        preContent: r.preContent ?? '',
      })
    }
    toolCallsByMessage.value = grouped
  }

  const lastUserMessageId = computed(() => {
    for (let i = messages.value.length - 1; i >= 0; i--) {
      if (messages.value[i].role === 'user') return messages.value[i].id
    }
    return 0
  })

  return { messages, toolCallsByMessage, lastUserMessageId, refreshMessages }
}

export function setConversationTitle(
  conversations: Ref<ConversationItem[]>,
  activeId: Ref<number>,
  activeTitle: Ref<string>,
  event: { conversationId: number; title: string },
) {
  const conv = conversations.value.find((c) => c.id === event.conversationId)
  if (conv) conv.title = event.title
  if (event.conversationId === activeId.value) activeTitle.value = event.title
}
