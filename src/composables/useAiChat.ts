/**
 * AI 聊天核心逻辑（组合式函数）
 *
 * 集中管理聊天状态与交互：
 * - 会话 CRUD、消息流式渲染（`ai-chat-stream` 事件逐 token 追加）
 * - 工具调用：`ai-ask-user` 弹窗提问、对话流内联展示技能调用（可点击查看入参/输出）
 * - 消息操作：删除（配对级联）/ 重新生成 / 编辑（仅最后一条用户消息，保存后自动重新生成）
 * - 模型选择与上下文窗口 token 估算
 * - 手动附加上下文前先选择游戏版本（版本隔离场景）
 */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useTauriEvent } from '@/composables/useTauriEvent'
import {
  experimentalCreateConversation,
  experimentalListConversations,
  experimentalDeleteConversation,
  experimentalListMessages,
  experimentalListToolCalls,
  experimentalClearConversation,
  experimentalChatSend,
  experimentalCollectContext,
  experimentalDeleteMessage,
  experimentalRegenerateReply,
  experimentalEditMessage,
  experimentalReplyAskUser,
  experimentalCancelChat,
  experimentalListInstalledVersions,
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

/** 工具调用条目（对话流中展示，可点击查看入参/输出） */
export interface ToolCallItem {
  index: string
  name: string
  status: 'running' | 'done'
  arguments: string
  output: string
  /** 调用该工具前模型输出的过渡文本 */
  preContent: string
}

/** 流式分帧参数（应对部分服务一次性返回完整内容，模拟打字机效果） */
const TYPE_FRAME_MS = 16
const TYPE_STEP = 12

export function useAiChat() {
  const conversations = ref<ConversationItem[]>([])
  const activeId = ref(0)
  const activeTitle = ref('')
  const messages = ref<LocalMessage[]>([])
  const inputText = ref('')
  /** 是否启用模型思考模式（关闭时后端透传 reasoning_effort=null） */
  const enableReasoning = ref(true)
  /** 思考程度（low/medium/high，映射滑块 0/50/100） */
  const reasoningLevel = ref<'low' | 'medium' | 'high'>('medium')
  /** 用户是否已手动上滑离开底部（自动滚动尊重用户意图） */
  const scrolledUp = ref(false)
  const loading = ref(false)
  const creating = ref(false)

  const models = ref<string[]>([])
  const currentModel = ref('')
  const maxInputTokens = ref(184000)
  const lastUsage = ref<AiChatStreamEvent['usage'] | null>(null)
  /** 本次回复的生成速度（t/s），done 事件到达时按耗时计算 */
  const chatSpeed = ref(0)
  /** 本次回复开始时间戳（ms），用于计算生成速度 */
  let chatStartedAt = 0
  /** 当前流式过程中的工具调用（实时 running/done，展示在流式消息上方） */
  const toolCalls = ref<ToolCallItem[]>([])
  /** 已持久化的工具链（按消息 id 分组；刷新/重启后从 SQLite 恢复） */
  const toolCallsByMessage = ref<Record<number, ToolCallItem[]>>({})
  /** 当前流式输出中的 AI 占位消息（独立于 messages，便于在工具调用条目之后渲染） */
  const streamingMsg = ref<LocalMessage | null>(null)

  const askUser = ref({ visible: false, question: '', options: [] as AskUserOption[] })
  const versionPicker = ref({ visible: false, versions: [] as string[] })
  const pendingAttachKind = ref('')

  /** 工具间隙等待中：正在流式但无正文/思考内容/工具执行（ask_user 等长等待场景） */
  const waitingNext = computed(
    () =>
      loading.value &&
      !!streamingMsg.value &&
      !streamingMsg.value.content &&
      !streamingMsg.value.reasoningContent &&
      toolCalls.value.length === 0,
  )
  /** 模型正在等待用户回答（ask_user 弹窗可见） */
  const waitingAsk = computed(() => askUser.value.visible)

  function setScrolledUp(v: boolean) {
    scrolledUp.value = v
  }

  // ---- 流式分帧（打字机） ----
  let deltaQueue = ''
  let typeTimer: number | null = null
  /** done 事件已到达但打字机队列尚未消费完：待消费完后统一收尾（清占位 + 刷新），
   * 防止密集流（delta 与 done 同批到达）被 done 清空队列、正文只能靠刷新一次性展示 */
  let donePending = false

  function finishStreaming() {
    flushStreaming()
    toolCalls.value = []
    void refreshMessages()
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
    if (typeTimer === null && streamingMsg.value) {
      typeTimer = window.setTimeout(typeNextFrame, TYPE_FRAME_MS)
    }
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

  const lastUserMessageId = computed(() => {
    for (let i = messages.value.length - 1; i >= 0; i--) {
      if (messages.value[i].role === 'user') return messages.value[i].id
    }
    return 0
  })

  /**
   * 上下文窗口"已用"估算：优先取最新一条 AI 消息的真实 usage.promptTokens
   * （该值即最近一次请求实际发送给模型的完整输入，含系统提示词/历史/工具定义/消息格式开销，
   * 比字符估算准确）；其后若还有未回复的用户消息则补估算。
   * 无真实 usage 时（新会话/旧数据）退化为前端估算，且计入思考内容。
   */
  const tokenEstimate = computed(() => {
    for (let i = messages.value.length - 1; i >= 0; i--) {
      const m = messages.value[i]
      if (m.role === 'assistant' && m.promptTokens && m.promptTokens > 0) {
        let extra = 0
        for (let j = i + 1; j < messages.value.length; j++) {
          extra += estimateTokens(messages.value[j].content) + estimateTokens(messages.value[j].reasoningContent)
        }
        return m.promptTokens + extra
      }
    }
    return messages.value.reduce((sum, m) => sum + estimateTokens(m.content) + estimateTokens(m.reasoningContent), 0)
  })
  const tokenPercent = computed(() =>
    maxInputTokens.value ? Math.min(100, Math.round((tokenEstimate.value / maxInputTokens.value) * 100)) : 0,
  )
  const tokenBarColor = computed(() => {
    if (tokenPercent.value >= 90) return 'bg-red-500'
    if (tokenPercent.value >= 70) return 'bg-amber-500'
    return 'bg-primary-500'
  })

  const modelOptions = computed(() => models.value.map((m) => ({ label: m, value: m })))

  const KIND_LABELS: Record<string, string> = {
    crash_report: '崩溃日志',
    game_logs: '游戏日志',
    mods: 'Mods 列表',
  }

  async function refreshMessages() {
    if (!activeId.value) return
    const [items, records] = await Promise.all([
      safeCall(() => experimentalListMessages(activeId.value), 'list messages', () => null),
      safeCall(() => experimentalListToolCalls(activeId.value), 'list tool calls', () => null),
    ])
    if (items) messages.value = items
    // 按消息 id 分组工具链（数据库持久化，刷新/重启后仍能展示）
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

  async function ensureConversation(): Promise<boolean> {
    if (activeId.value) return true
    creating.value = true
    const conv = await safeCall(() => experimentalCreateConversation('新对话'), 'create conversation', () => null)
    creating.value = false
    if (!conv) {
      toastError('创建会话失败')
      return false
    }
    conversations.value.unshift(conv)
    activeId.value = conv.id
    activeTitle.value = conv.title
    return true
  }

  async function selectConversation(id: number) {
    setScrolledUp(false)
    if (id !== activeId.value) {
      flushStreaming()
      loading.value = false
      toolCalls.value = []
      toolCallsByMessage.value = {}
    }
    activeId.value = id
    const conv = conversations.value.find((c) => c.id === id)
    activeTitle.value = conv?.title ?? ''
    lastUsage.value = null
    await refreshMessages()
  }

  async function newConversation() {
    if (creating.value || loading.value) return
    creating.value = true
    const conv = await safeCall(() => experimentalCreateConversation('新对话'), 'create conversation', () => null)
    creating.value = false
    if (!conv) {
      toastError('创建会话失败')
      return
    }
    conversations.value.unshift(conv)
    activeId.value = conv.id
    activeTitle.value = conv.title
    messages.value = []
    inputText.value = ''
    flushStreaming()
    toolCalls.value = []
    toolCallsByMessage.value = {}
    lastUsage.value = null
  }

  async function removeConversation(conv: ConversationItem) {
    if (loading.value) return
    const ok = await showConfirmAsync('删除会话', `确定删除会话「${conv.title}」？`)
    if (!ok) return
    const res = await safeCall(() => experimentalDeleteConversation(conv.id), 'delete conversation', () => null)
    if (res !== undefined) {
      conversations.value = conversations.value.filter((c) => c.id !== conv.id)
      if (activeId.value === conv.id) {
        activeId.value = 0
        activeTitle.value = ''
        messages.value = []
        toolCallsByMessage.value = {}
      }
      toastSuccess('会话已删除')
    } else {
      toastError('删除会话失败')
    }
  }

  async function clearMessages() {
    if (!activeId.value || loading.value) return
    const ok = await showConfirmAsync('清空消息', '确定清空当前会话的所有消息？')
    if (!ok) return
    const res = await safeCall(() => experimentalClearConversation(activeId.value), 'clear conversation', () => null)
    if (res !== undefined) {
      messages.value = []
      flushStreaming()
      toolCalls.value = []
      toolCallsByMessage.value = {}
      lastUsage.value = null
      toastSuccess('已清空')
    } else {
      toastError('清空失败')
    }
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
      model: currentModel.value || null,
      streaming: true,
    }
  }

  async function handleStreamFailed() {
    loading.value = false
    flushStreaming()
    toolCalls.value = []
    await refreshMessages()
    toastError('AI 请求失败，请检查服务配置或稍后重试')
  }

  async function send() {
    const content = inputText.value.trim()
    if (!content) {
      toastError('请输入消息内容')
      return
    }
    if (loading.value) return
    if (!(await ensureConversation())) return
    setScrolledUp(false)
    loading.value = true
    chatStartedAt = Date.now()
    chatSpeed.value = 0
    lastUsage.value = null
    toolCalls.value = []
    messages.value.push({ id: 0, role: 'user', content, createdAt: Math.floor(Date.now() / 1000), pairId: null, versionId: null })
    startStreaming()
    inputText.value = ''
    const result = await safeCall(
      () =>
        experimentalChatSend({
          conversationId: activeId.value,
          content,
          model: currentModel.value || undefined,
          reasoningEffort: enableReasoning.value ? reasoningLevel.value : null,
        }),
      'chat send',
      () => null,
    )
    if (!result) await handleStreamFailed()
  }

  async function regenerate(msg: LocalMessage) {
    if (!activeId.value || loading.value || msg.id === 0) return
    const idx = messages.value.indexOf(msg)
    if (idx < 0) return
    setScrolledUp(false)
    loading.value = true
    chatStartedAt = Date.now()
    chatSpeed.value = 0
    lastUsage.value = null
    toolCalls.value = []
    messages.value = messages.value.slice(0, idx)
    // 重新生成：流式占位即显示「第 N 次重试」（旧回复序号 + 1）
    startStreaming((msg.retryCount ?? 1) + 1)
    const result = await safeCall(
      () => experimentalRegenerateReply(activeId.value, msg.id, currentModel.value || undefined, enableReasoning.value ? reasoningLevel.value : null),
      'regenerate reply',
      () => null,
    )
    if (!result) await handleStreamFailed()
  }

  async function editMessage(msg: LocalMessage, content: string) {
    if (!activeId.value || loading.value || msg.id === 0) return
    const idx = messages.value.indexOf(msg)
    if (idx < 0) return
    setScrolledUp(false)
    loading.value = true
    lastUsage.value = null
    toolCalls.value = []
    msg.content = content
    messages.value = messages.value.slice(0, idx + 1)
    startStreaming()
    const result = await safeCall(
      () => experimentalEditMessage(activeId.value, msg.id, content, currentModel.value || undefined, enableReasoning.value ? reasoningLevel.value : null),
      'edit message',
      () => null,
    )
    if (!result) await handleStreamFailed()
  }

  async function deleteMessage(msg: LocalMessage) {
    if (!activeId.value || loading.value || msg.id === 0) return
    const tip = msg.role === 'user' ? '将同时删除 AI 的回复消息，确定删除？' : '将同时删除对应的用户消息，确定删除？'
    const ok = await showConfirmAsync('删除消息', tip)
    if (!ok) return
    const res = await safeCall(() => experimentalDeleteMessage(activeId.value, msg.id), 'delete message', () => null)
    if (res !== undefined) {
      await refreshMessages()
      toastSuccess('消息已删除')
    } else {
      toastError('删除消息失败')
    }
  }

  async function attach(kind: string) {
    if (!activeId.value || loading.value) return
    pendingAttachKind.value = kind
    const versions = await safeCall(() => experimentalListInstalledVersions(), 'list versions', () => null)
    if (versions && versions.length > 0) {
      versionPicker.value = { visible: true, versions }
    } else {
      await doCollect(kind)
    }
  }

  async function doCollect(kind: string, versionId?: string) {
    versionPicker.value = { visible: false, versions: [] }
    const result = await safeCall(
      () => experimentalCollectContext({ kind, versionId, conversationId: activeId.value }),
      'collect context',
      () => null,
    )
    if (!result) {
      toastError('收集上下文失败')
      return
    }
    const label = KIND_LABELS[kind] ?? kind
    const block = `【自动收集：${label}${versionId ? `（${versionId}）` : ''}】\n${result.text}\n\n---\n\n`
    inputText.value = inputText.value.trim() ? `${block}${inputText.value}` : block
    toastInfo(`已收集 ${label} 内容到输入框，检查后发送`)
  }

  async function submitAskUser(reply: string) {
    askUser.value.visible = false
    const ok = await safeCall(() => experimentalReplyAskUser(activeId.value, reply), 'reply ask user', () => null)
    if (ok === undefined) toastError('提交回答失败')
  }

  async function cancelAskUser() {
    askUser.value.visible = false
    await safeCall(() => experimentalReplyAskUser(activeId.value, '用户取消了提问'), 'reply ask user', () => null)
  }

  /** 暂停当前流式回复：置位后端取消信号，流式输出尽快中断 */
  async function cancel() {
    if (!loading.value) return
    await safeCall(() => experimentalCancelChat(), 'cancel chat', () => null)
  }

  function onEnterKey(e: KeyboardEvent) {
    if (e.isComposing) return
    if (!e.shiftKey) {
      e.preventDefault()
      void send()
    }
  }

  // ---- 事件监听（组件挂载后自动注册） ----
  const streamEvent = useTauriEvent<AiChatStreamEvent>('ai-chat-stream', (ev) => {
    if (ev.conversationId !== activeId.value) return
    if (ev.status) {
      toastInfo(ev.status)
      return
    }
    if (ev.toolCall) {
      const tc = ev.toolCall
      const idx = tc.index ?? tc.name
      const isFirstRunning = tc.status === 'running' && toolCalls.value.length === 0
      const existing = toolCalls.value.find((t) => t.index === idx)
      if (tc.status === 'running') {
        if (isFirstRunning) {
          // 首个工具开始调用：清空此前流式输出的过渡语句（如「我来读取…」），
          // 该过渡文本已随 running 事件的 preContent 交给工具链展示
          if (streamingMsg.value) streamingMsg.value.content = ''
          deltaQueue = ''
        }
        if (existing) {
          if (tc.arguments !== undefined) existing.arguments = tc.arguments
          if (tc.preContent !== undefined) existing.preContent = tc.preContent
        } else {
          toolCalls.value.push({
            index: idx,
            name: tc.name,
            status: 'running',
            arguments: tc.arguments ?? '',
            output: '',
            preContent: tc.preContent ?? '',
          })
        }
      } else if (existing) {
        existing.status = 'done'
        if (tc.output !== undefined) existing.output = tc.output
      } else {
        toolCalls.value.push({
          index: idx,
          name: tc.name,
          status: 'done',
          arguments: tc.arguments ?? '',
          output: tc.output ?? '',
          preContent: tc.preContent ?? '',
        })
      }
      return
    }
    // 思考内容增量（思考模型，如 DeepSeek-R1）：直接追加到流式消息，由「深度思考」区块实时渲染
    if (ev.reasoning) {
      if (streamingMsg.value) {
        streamingMsg.value.reasoningContent = (streamingMsg.value.reasoningContent ?? '') + ev.reasoning
      }
      return
    }
    if (ev.delta) {
      pushDelta(ev.delta)
      return
    }
    if (ev.done) {
      lastUsage.value = ev.usage ?? null
      // 生成速度：输出 token（completion）÷ 总耗时（含工具调用轮次），与中转站口径一致；
      // 优先用后端返回的 durationMs，兜底用前端计时的总耗时
      const completion = ev.usage?.completionTokens ?? 0
      if (completion > 0) {
        const ms = ev.durationMs ?? Date.now() - chatStartedAt
        chatSpeed.value = ms > 0 ? Math.round((completion * 1000) / ms) : 0
      } else {
        chatSpeed.value = 0
      }
      chatStartedAt = 0
      loading.value = false
      // delta 与 done 可能同批到达（短回复 + 密集流）：让打字机先把队列消费完再收尾，
      // 保证正文逐字实时显示，而不是被 done 清空队列后靠刷新一次性展示
      if (deltaQueue && streamingMsg.value) {
        donePending = true
        if (typeTimer === null) {
          typeTimer = window.setTimeout(typeNextFrame, TYPE_FRAME_MS)
        }
      } else {
        finishStreaming()
      }
    }
  })

  const askUserEvent = useTauriEvent<AiAskUserEvent>('ai-ask-user', (ev) => {
    if (ev.conversationId !== activeId.value) return
    // 兼容纯字符串选项（旧后端/兜底），统一归一化为 {label, description?}
    const options: AskUserOption[] = (ev.options ?? []).map((o) =>
      typeof o === 'string' ? { label: o } : o,
    )
    askUser.value = { visible: true, question: ev.question, options }
  })

  const titleEvent = useTauriEvent<{ conversationId: number; title: string }>('conversation-title-updated', (ev) => {
    const conv = conversations.value.find((c) => c.id === ev.conversationId)
    if (conv) conv.title = ev.title
    if (ev.conversationId === activeId.value) activeTitle.value = ev.title
  })

  onMounted(async () => {
    const cfg = await safeCall(() => aiLoadConfig(), 'load ai config', () => null)
    if (cfg) {
      models.value = cfg.models ?? []
      currentModel.value = cfg.defaultModel || cfg.models?.[0] || ''
      maxInputTokens.value = cfg.maxInputTokens ?? 184000
      setIconColorMode(cfg.iconColorMode)
    }
    streamEvent.start()
    askUserEvent.start()
    titleEvent.start()
    const items = await safeCall(() => experimentalListConversations(), 'list conversations', () => null)
    conversations.value = items ?? []
    if (conversations.value.length > 0) {
      await selectConversation(conversations.value[0].id)
    } else {
      await ensureConversation()
    }
  })

  onUnmounted(() => {
    flushStreaming()
  })

  return {
    conversations,
    activeId,
    activeTitle,
    messages,
    inputText,
    enableReasoning,
    reasoningLevel,
    waitingNext,
    waitingAsk,
    scrolledUp,
    setScrolledUp,
    loading,
    creating,
    models,
    currentModel,
    maxInputTokens,
    lastUsage,
    chatSpeed,
    toolCalls,
    toolCallsByMessage,
    streamingMsg,
    askUser,
    versionPicker,
    pendingAttachKind,
    lastUserMessageId,
    tokenEstimate,
    tokenPercent,
    tokenBarColor,
    modelOptions,
    formatTokens,
    selectConversation,
    newConversation,
    removeConversation,
    clearMessages,
    send,
    regenerate,
    editMessage,
    deleteMessage,
    attach,
    doCollect,
    submitAskUser,
    cancelAskUser,
    cancel,
    onEnterKey,
  }
}
