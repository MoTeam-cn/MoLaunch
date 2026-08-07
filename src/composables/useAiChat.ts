/**
 * AI 聊天入口组合式函数：组装会话、消息、流式事件和界面状态。
 * 具体职责分别位于 aiChatMessages / aiChatStream，保持原有调用 API 不变。
 */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useTauriEvent } from '@/composables/useTauriEvent'
import {
  experimentalCreateConversation,
  experimentalListConversations,
  experimentalDeleteConversation,
  experimentalClearConversation,
  experimentalDeleteMessage,
  experimentalReplyAskUser,
  experimentalCancelChat,
  experimentalListInstalledVersions,
  experimentalCollectContext,
  type ConversationItem,
  type AiChatStreamEvent,
  type AiAskUserEvent,
  type AskUserOption,
} from '@/utils/api/experimental'
import { aiLoadConfig } from '@/utils/api/ai'
import { safeCall } from '@/utils/async'
import { setIconColorMode } from '@/utils/model-icon-mode'
import { toastError, toastSuccess, toastInfo } from '@/utils/toast'
import { estimateTokens, formatTokens } from '@/utils/tokens'
import { showConfirmAsync } from '@/utils/modal'
import type { LocalMessage } from '@/components/experimental/ChatMessageItem.vue'
import { useAiChatMessages, setConversationTitle } from './aiChatMessages'
import { useAiChatStream } from './aiChatStream'
import type { ToolCallItem } from './aiChatTypes'

export type { ToolCallItem }

const KIND_LABELS: Record<string, string> = {
  crash_report: '崩溃日志',
  game_logs: '游戏日志',
  mods: 'Mods 列表',
}

export function useAiChat() {
  const conversations = ref<ConversationItem[]>([])
  const activeId = ref(0)
  const activeTitle = ref('')
  const inputText = ref('')
  const enableReasoning = ref(true)
  const reasoningLevel = ref<'low' | 'medium' | 'high'>('medium')
  const scrolledUp = ref(false)
  const loading = ref(false)
  const creating = ref(false)
  const models = ref<string[]>([])
  const currentModel = ref('')
  const maxInputTokens = ref(184000)
  const lastUsage = ref<AiChatStreamEvent['usage'] | null>(null)
  const chatSpeed = ref(0)
  const toolCalls = ref<ToolCallItem[]>([])
  const askUser = ref({ visible: false, question: '', options: [] as AskUserOption[] })
  const versionPicker = ref({ visible: false, versions: [] as string[] })
  const pendingAttachKind = ref('')

  const { messages, toolCallsByMessage, lastUserMessageId, refreshMessages } = useAiChatMessages(activeId)
  const stream = useAiChatStream({
    activeId, messages, loading, currentModel, enableReasoning, reasoningLevel, lastUsage, chatSpeed, toolCalls,
    refreshMessages, setScrolledUp: (value) => (scrolledUp.value = value),
  })
  const streamingMsg = stream.streamingMsg
  const waitingAsk = computed(() => askUser.value.visible)

  function setScrolledUp(value: boolean) { scrolledUp.value = value }

  const tokenEstimate = computed(() => {
    for (let i = messages.value.length - 1; i >= 0; i--) {
      const message = messages.value[i]
      if (message.role === 'assistant' && message.promptTokens && message.promptTokens > 0) {
        let extra = 0
        for (let j = i + 1; j < messages.value.length; j++) {
          extra += estimateTokens(messages.value[j].content) + estimateTokens(messages.value[j].reasoningContent)
        }
        return message.promptTokens + extra
      }
    }
    return messages.value.reduce((sum, message) => sum + estimateTokens(message.content) + estimateTokens(message.reasoningContent), 0)
  })
  const tokenPercent = computed(() => maxInputTokens.value ? Math.min(100, Math.round((tokenEstimate.value / maxInputTokens.value) * 100)) : 0)
  const tokenBarColor = computed(() => tokenPercent.value >= 90 ? 'bg-red-500' : tokenPercent.value >= 70 ? 'bg-amber-500' : 'bg-primary-500')
  const modelOptions = computed(() => models.value.map((model) => ({ label: model, value: model })))

  async function ensureConversation() {
    if (activeId.value) return true
    creating.value = true
    const conv = await safeCall(() => experimentalCreateConversation('新对话'), 'create conversation', () => null)
    creating.value = false
    if (!conv) { toastError('创建会话失败'); return false }
    conversations.value.unshift(conv)
    activeId.value = conv.id
    activeTitle.value = conv.title
    return true
  }

  async function selectConversation(id: number) {
    setScrolledUp(false)
    if (id !== activeId.value) {
      stream.flushStreaming(); loading.value = false; toolCalls.value = []; toolCallsByMessage.value = {}
    }
    activeId.value = id
    activeTitle.value = conversations.value.find((conversation) => conversation.id === id)?.title ?? ''
    lastUsage.value = null
    await refreshMessages()
  }

  async function newConversation() {
    if (creating.value || loading.value) return
    creating.value = true
    const conv = await safeCall(() => experimentalCreateConversation('新对话'), 'create conversation', () => null)
    creating.value = false
    if (!conv) { toastError('创建会话失败'); return }
    conversations.value.unshift(conv); activeId.value = conv.id; activeTitle.value = conv.title
    messages.value = []; inputText.value = ''; stream.flushStreaming(); toolCalls.value = []; toolCallsByMessage.value = {}; lastUsage.value = null
  }

  async function removeConversation(conv: ConversationItem) {
    if (loading.value || !(await showConfirmAsync('删除会话', `确定删除会话「${conv.title}」？`))) return
    const result = await safeCall(() => experimentalDeleteConversation(conv.id), 'delete conversation', () => null)
    if (result !== undefined) {
      conversations.value = conversations.value.filter((item) => item.id !== conv.id)
      if (activeId.value === conv.id) { activeId.value = 0; activeTitle.value = ''; messages.value = []; toolCallsByMessage.value = {} }
      toastSuccess('会话已删除')
    } else toastError('删除会话失败')
  }

  async function clearMessages() {
    if (!activeId.value || loading.value || !(await showConfirmAsync('清空消息', '确定清空当前会话的所有消息？'))) return
    const result = await safeCall(() => experimentalClearConversation(activeId.value), 'clear conversation', () => null)
    if (result !== undefined) { messages.value = []; stream.flushStreaming(); toolCalls.value = []; toolCallsByMessage.value = {}; lastUsage.value = null; toastSuccess('已清空') }
    else toastError('清空失败')
  }

  async function send() {
    const content = inputText.value.trim()
    if (!content) { toastError('请输入消息内容'); return }
    if (loading.value || !(await ensureConversation())) return
    inputText.value = ''
    await stream.send(content)
  }

  async function regenerate(message: LocalMessage) {
    if (!activeId.value || loading.value || message.id === 0) return
    const index = messages.value.indexOf(message)
    if (index < 0) return
    setScrolledUp(false)
    await stream.regenerate(message)
  }

  async function editMessage(message: LocalMessage, content: string) {
    if (!activeId.value || loading.value || message.id === 0) return
    if (messages.value.indexOf(message) < 0) return
    setScrolledUp(false)
    await stream.editMessage(message, content)
  }

  async function deleteMessage(message: LocalMessage) {
    if (!activeId.value || loading.value || message.id === 0) return
    const tip = message.role === 'user' ? '将同时删除 AI 的回复消息，确定删除？' : '将同时删除对应的用户消息，确定删除？'
    if (!(await showConfirmAsync('删除消息', tip))) return
    const result = await safeCall(() => experimentalDeleteMessage(activeId.value, message.id), 'delete message', () => null)
    if (result !== undefined) { await refreshMessages(); toastSuccess('消息已删除') } else toastError('删除消息失败')
  }

  async function attach(kind: string) {
    if (!activeId.value || loading.value) return
    pendingAttachKind.value = kind
    const versions = await safeCall(() => experimentalListInstalledVersions(), 'list versions', () => null)
    if (versions && versions.length > 0) versionPicker.value = { visible: true, versions }
    else await doCollect(kind)
  }

  async function doCollect(kind: string, versionId?: string) {
    versionPicker.value = { visible: false, versions: [] }
    const result = await safeCall(() => experimentalCollectContext({ kind, versionId, conversationId: activeId.value }), 'collect context', () => null)
    if (!result) { toastError('收集上下文失败'); return }
    const label = KIND_LABELS[kind] ?? kind
    const block = `【自动收集：${label}${versionId ? `（${versionId}）` : ''}】\n${result.text}\n\n---\n\n`
    inputText.value = inputText.value.trim() ? `${block}${inputText.value}` : block
    toastInfo(`已收集 ${label} 内容到输入框，检查后发送`)
  }

  async function submitAskUser(reply: string) {
    askUser.value.visible = false
    const result = await safeCall(() => experimentalReplyAskUser(activeId.value, reply), 'reply ask user', () => null)
    if (result === undefined) toastError('提交回答失败')
  }

  async function cancelAskUser() {
    askUser.value.visible = false
    await safeCall(() => experimentalReplyAskUser(activeId.value, '用户取消了提问'), 'reply ask user', () => null)
  }

  async function cancel() {
    if (loading.value) await safeCall(() => experimentalCancelChat(), 'cancel chat', () => null)
  }

  function onEnterKey(event: KeyboardEvent) {
    if (event.isComposing) return
    if (!event.shiftKey) { event.preventDefault(); void send() }
  }

  const streamEvent = useTauriEvent<AiChatStreamEvent>('ai-chat-stream', (event) => {
    if (event.conversationId !== activeId.value) return
    if (event.status) { toastInfo(event.status); return }
    stream.handleEvent(event)
  })
  const askUserEvent = useTauriEvent<AiAskUserEvent>('ai-ask-user', (event) => {
    if (event.conversationId !== activeId.value) return
    askUser.value = { visible: true, question: event.question, options: (event.options ?? []).map((option) => typeof option === 'string' ? { label: option } : option) }
  })
  const titleEvent = useTauriEvent<{ conversationId: number; title: string }>('conversation-title-updated', (event) => {
    setConversationTitle(conversations, activeId, activeTitle, event)
  })

  onMounted(async () => {
    const config = await safeCall(() => aiLoadConfig(), 'load ai config', () => null)
    if (config) { models.value = config.models ?? []; currentModel.value = config.defaultModel || config.models?.[0] || ''; maxInputTokens.value = config.maxInputTokens ?? 184000; setIconColorMode(config.iconColorMode) }
    streamEvent.start(); askUserEvent.start(); titleEvent.start()
    conversations.value = (await safeCall(() => experimentalListConversations(), 'list conversations', () => null)) ?? []
    if (conversations.value.length > 0) await selectConversation(conversations.value[0].id)
    else await ensureConversation()
  })

  onUnmounted(() => stream.flushStreaming())

  return { conversations, activeId, activeTitle, messages, inputText, enableReasoning, reasoningLevel, waitingAsk, scrolledUp, setScrolledUp, loading, creating, models, currentModel, maxInputTokens, lastUsage, chatSpeed, toolCalls, toolCallsByMessage, streamingMsg, askUser, versionPicker, pendingAttachKind, lastUserMessageId, tokenEstimate, tokenPercent, tokenBarColor, modelOptions, formatTokens, selectConversation, newConversation, removeConversation, clearMessages, send, regenerate, editMessage, deleteMessage, attach, doCollect, submitAskUser, cancelAskUser, cancel, onEnterKey }
}
