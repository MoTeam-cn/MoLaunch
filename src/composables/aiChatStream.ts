import { ref, type Ref } from 'vue'
import {
  experimentalChatSend,
  experimentalRegenerateReply,
  experimentalEditMessage,
  type AiChatStreamEvent,
} from '@/utils/api/experimental'
import { safeCall } from '@/utils/async'
import { toastError } from '@/utils/toast'
import type { LocalMessage } from '@/components/experimental/ChatMessageItem.vue'
import type { ToolCallItem } from './aiChatTypes'

const TYPE_FRAME_MS = 16
const TYPE_STEP = 12

export function useAiChatStream(state: {
  activeId: Ref<number>
  messages: Ref<LocalMessage[]>
  loading: Ref<boolean>
  currentModel: Ref<string>
  enableReasoning: Ref<boolean>
  reasoningLevel: Ref<'low' | 'medium' | 'high'>
  lastUsage: Ref<AiChatStreamEvent['usage'] | null>
  chatSpeed: Ref<number>
  toolCalls: Ref<ToolCallItem[]>
  refreshMessages: () => Promise<void>
  setScrolledUp: (value: boolean) => void
}) {
  const streamingMsg = ref<LocalMessage | null>(null)
  let deltaQueue = ''
  let typeTimer: number | null = null
  let donePending = false
  let chatStartedAt = 0

  function finishStreaming() {
    flushStreaming()
    state.toolCalls.value = []
    void state.refreshMessages()
  }

  function typeNextFrame() {
    if (!streamingMsg.value) {
      typeTimer = null
      if (donePending) {
        donePending = false
        finishStreaming()
      }
      return
    }
    const slice = deltaQueue.slice(0, TYPE_STEP)
    if (slice) {
      deltaQueue = deltaQueue.slice(TYPE_STEP)
      streamingMsg.value.content += slice
      typeTimer = window.setTimeout(typeNextFrame, TYPE_FRAME_MS)
    } else {
      typeTimer = null
      if (donePending) {
        donePending = false
        finishStreaming()
      }
    }
  }

  function pushDelta(delta: string) {
    deltaQueue += delta
    if (typeTimer === null && streamingMsg.value) typeTimer = window.setTimeout(typeNextFrame, TYPE_FRAME_MS)
  }

  function flushStreaming() {
    deltaQueue = ''
    if (typeTimer !== null) {
      clearTimeout(typeTimer)
      typeTimer = null
    }
    donePending = false
    streamingMsg.value = null
  }

  function startStreaming(retryCount?: number) {
    flushStreaming()
    streamingMsg.value = {
      id: 0,
      role: 'assistant',
      content: '',
      createdAt: Math.floor(Date.now() / 1000),
      pairId: null,
      versionId: null,
      retryCount,
      model: state.currentModel.value || null,
      streaming: true,
    }
  }

  async function handleStreamFailed() {
    state.loading.value = false
    flushStreaming()
    state.toolCalls.value = []
    await state.refreshMessages()
    toastError('AI 请求失败，请检查服务配置或稍后重试')
  }

  function beginRequest() {
    state.setScrolledUp(false)
    state.loading.value = true
    chatStartedAt = Date.now()
    state.chatSpeed.value = 0
    state.lastUsage.value = null
    state.toolCalls.value = []
  }

  async function send(content: string) {
    beginRequest()
    state.messages.value.push({ id: 0, role: 'user', content, createdAt: Math.floor(Date.now() / 1000), pairId: null, versionId: null })
    startStreaming()
    const result = await safeCall(
      () => experimentalChatSend({
        conversationId: state.activeId.value,
        content,
        model: state.currentModel.value || undefined,
        reasoningEffort: state.enableReasoning.value ? state.reasoningLevel.value : null,
      }),
      'chat send',
      () => null,
    )
    if (!result) await handleStreamFailed()
  }

  async function regenerate(msg: LocalMessage) {
    beginRequest()
    const idx = state.messages.value.indexOf(msg)
    state.messages.value = state.messages.value.slice(0, idx)
    startStreaming((msg.retryCount ?? 1) + 1)
    const result = await safeCall(
      () => experimentalRegenerateReply(state.activeId.value, msg.id, state.currentModel.value || undefined, state.enableReasoning.value ? state.reasoningLevel.value : null),
      'regenerate reply',
      () => null,
    )
    if (!result) await handleStreamFailed()
  }

  async function editMessage(msg: LocalMessage, content: string) {
    beginRequest()
    const idx = state.messages.value.indexOf(msg)
    msg.content = content
    state.messages.value = state.messages.value.slice(0, idx + 1)
    startStreaming()
    const result = await safeCall(
      () => experimentalEditMessage(state.activeId.value, msg.id, content, state.currentModel.value || undefined, state.enableReasoning.value ? state.reasoningLevel.value : null),
      'edit message',
      () => null,
    )
    if (!result) await handleStreamFailed()
  }

  function handleEvent(ev: AiChatStreamEvent) {
    if (ev.status) return
    if (ev.toolCall) {
      const tc = ev.toolCall
      const idx = tc.index ?? tc.name
      const existing = state.toolCalls.value.find((t) => t.index === idx)
      if (tc.status === 'running') {
        if (streamingMsg.value) {
          streamingMsg.value.content = ''
          streamingMsg.value.reasoningContent = ''
        }
        deltaQueue = ''
        if (existing) {
          if (tc.arguments !== undefined) existing.arguments = tc.arguments
          if (tc.preContent !== undefined) existing.preContent = tc.preContent
        } else {
          state.toolCalls.value.push({ index: idx, name: tc.name, status: 'running', arguments: tc.arguments ?? '', output: '', preContent: tc.preContent ?? '' })
        }
      } else if (existing) {
        existing.status = 'done'
        if (tc.output !== undefined) existing.output = tc.output
      } else {
        state.toolCalls.value.push({ index: idx, name: tc.name, status: 'done', arguments: tc.arguments ?? '', output: tc.output ?? '', preContent: tc.preContent ?? '' })
      }
      return
    }
    if (ev.reasoning) {
      if (streamingMsg.value) streamingMsg.value.reasoningContent = (streamingMsg.value.reasoningContent ?? '') + ev.reasoning
      return
    }
    if (ev.delta) {
      pushDelta(ev.delta)
      return
    }
    if (ev.done) {
      state.lastUsage.value = ev.usage ?? null
      const completion = ev.usage?.completionTokens ?? 0
      if (completion > 0) {
        const ms = ev.durationMs ?? Date.now() - chatStartedAt
        state.chatSpeed.value = ms > 0 ? Math.round((completion * 1000) / ms) : 0
      } else state.chatSpeed.value = 0
      chatStartedAt = 0
      state.loading.value = false
      if (deltaQueue && streamingMsg.value) {
        donePending = true
        if (typeTimer === null) typeTimer = window.setTimeout(typeNextFrame, TYPE_FRAME_MS)
      } else finishStreaming()
    }
  }

  return { streamingMsg, flushStreaming, send, regenerate, editMessage, startStreaming, handleEvent }
}
