<script setup lang="ts">
/**
 * AI 聊天（实验性）主视图
 *
 * 仅负责布局编排，全部逻辑集中在 useAiChat() 组合式函数：
 * - 左侧：会话列表（ChatConversationList）
 * - 右侧：头部（ChatHeader，含模型选择/进度条）、消息列表（含技能调用条目）、输入区
 * - 弹窗：AskUserDialog（工具 ask_user 提问）、VersionPickerDialog（版本隔离开关下先选版本）
 */
import { computed, ref, watch, defineAsyncComponent } from 'vue'
import { CommandLineIcon, PaperAirplaneIcon, PaperClipIcon, PauseIcon, Squares2X2Icon, ChatBubbleLeftRightIcon } from '@heroicons/vue/24/outline'
import { useAiChat } from '@/composables/useAiChat'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const ToolToc = defineAsyncComponent(() => import('@/components/common/ToolToc.vue'))
const ChatConversationList = defineAsyncComponent(() => import('@/components/experimental/ChatConversationList.vue'))
const ChatHeader = defineAsyncComponent(() => import('@/components/experimental/ChatHeader.vue'))
import ChatMessageItem, { type LocalMessage } from '@/components/experimental/ChatMessageItem.vue'
const ToolCallEntry = defineAsyncComponent(() => import('@/components/experimental/ToolCallEntry.vue'))
const AskUserDialog = defineAsyncComponent(() => import('@/components/experimental/AskUserDialog.vue'))
const VersionPickerDialog = defineAsyncComponent(() => import('@/components/experimental/VersionPickerDialog.vue'))
import { markdownToPlainText } from '@/utils/markdown'

const c = useAiChat()

const listRef = ref<HTMLElement | null>(null)

/** 滚动监听：用户上滑离开底部时置位（自动滚动尊重用户意图） */
function onScroll() {
  const el = listRef.value
  if (el) c.setScrolledUp(el.scrollHeight - el.scrollTop - el.clientHeight > 64)
}

/** TOC 目录条目标题：取用户消息内容前 15 字预览（会话目录直接使用用户消息，无需模型生成摘要） */
function tocTitle(msg: LocalMessage): string {
  const plain = markdownToPlainText(msg.content).trim().replace(/\s+/g, ' ')
  return plain.slice(0, 15) || '提问'
}

/** 触发 TOC 重新扫描：会话或消息集合变化时（含流式完成后刷新） */
const tocRefreshKey = computed(() => {
  const msgs = c.messages.value
  const last = msgs[msgs.length - 1]
  return c.activeId.value + ':' + msgs.length + ':' + (last ? last.id : 0)
})

watch(
  () => {
    const msgs = c.messages.value
    const last = msgs[msgs.length - 1]
    return (
      msgs.length +
      ':' +
      (last ? last.content.length : 0) +
      ':tc' +
      c.toolCalls.value.length +
      ':sm' +
      (c.streamingMsg.value ? c.streamingMsg.value.content.length : 0) +
      ':sr' +
      (c.streamingMsg.value?.reasoningContent?.length ?? 0)
    )
  },
  async () => {
    await new Promise((r) => setTimeout(r, 30))
    // 仅当用户未手动上滑时自动滚底（send/regenerate/edit 时 useAiChat 已重置标记）
    if (listRef.value && !c.scrolledUp.value) listRef.value.scrollTop = listRef.value.scrollHeight
  },
)
</script>

<template>
  <div class="relative flex h-full min-h-0">
    <ChatConversationList
      :conversations="c.conversations.value"
      :active-id="c.activeId.value"
      :creating="c.creating.value"
      @select="c.selectConversation"
      @create="c.newConversation"
      @remove="c.removeConversation"
    />

    <section class="flex min-w-0 flex-1 flex-col">
      <ChatHeader
        :title="c.activeTitle.value"
        :active-id="c.activeId.value"
        :loading="c.loading.value"
        :models="c.models.value"
        :current-model="c.currentModel.value"
        :enable-reasoning="c.enableReasoning.value"
        :reasoning-level="c.reasoningLevel.value"
        @update:current-model="c.currentModel.value = $event"
        @update:enable-reasoning="c.enableReasoning.value = $event"
        @update:reasoning-level="c.reasoningLevel.value = $event as 'low' | 'medium' | 'high'"
        @clear="c.clearMessages"
      />

      <!-- 消息列表（data-inner-scroll：内部滚动，不触发全局返回顶部按钮） -->
      <div class="relative min-h-0 flex-1">
        <div
          id="experimental-chat-scroll"
          ref="listRef"
          data-inner-scroll
          class="h-full overflow-y-auto px-4 py-4"
          @scroll="onScroll"
        >
          <!-- 会话切换过渡：旧会话内容淡出，新会话淡入 -->
          <Transition name="conv-view" mode="out-in">
            <div :key="c.activeId.value" class="h-full">
              <div
                v-if="c.messages.value.length === 0"
                class="flex h-full flex-col items-center justify-center text-gray-400"
              >
                <ChatBubbleLeftRightIcon class="mb-2 h-10 w-10" />
                <span class="text-sm">开始一段对话吧，可以直接输入消息</span>
              </div>
              <div v-else class="space-y-3">
                <template v-for="(msg, i) in c.messages.value" :key="`${msg.id}-${i}`">
                  <!-- 工具链：位于 AI 回复消息框上方（SQLite 持久化，刷新/重启后仍保留） -->
                  <ToolCallEntry
                    v-if="msg.role === 'assistant' && c.toolCallsByMessage.value[msg.id]?.length"
                    :calls="c.toolCallsByMessage.value[msg.id]"
                  />
                  <ChatMessageItem
                    :id="msg.role === 'user' && msg.id > 0 ? `msg-${msg.id}` : undefined"
                    :data-toc-card="msg.role === 'user' && msg.id > 0 ? `msg-${msg.id}` : undefined"
                    :data-toc-title="msg.role === 'user' && msg.id > 0 ? tocTitle(msg) : undefined"
                    :message="msg"
                    :model="c.currentModel.value || null"
                    :busy="c.loading.value"
                    :is-last-user="msg.role === 'user' && msg.id === c.lastUserMessageId.value"
                    @delete="c.deleteMessage(msg)"
                    @regenerate="c.regenerate(msg)"
                    @edit="(content: string) => c.editMessage(msg, content)"
                  />
                </template>
                <ToolCallEntry
                  v-if="c.streamingMsg.value && c.toolCalls.value.length > 0"
                  :calls="c.toolCalls.value"
                  :auto-expand="true"
                />
                <!-- 流式输出占位（独立于 messages，渲染在消息流末尾） -->
                <ChatMessageItem
                  v-if="c.streamingMsg.value"
                  :message="c.streamingMsg.value"
                  :model="c.currentModel.value || null"
                  :busy="c.loading.value"
                  :waiting="
                    c.loading.value &&
                    !!c.streamingMsg.value &&
                    (c.waitingAsk.value || c.toolCalls.value.length > 0)
                  "
                  :live-speed="c.chatSpeed.value"
                  :live-completion="c.lastUsage.value?.completionTokens"
                />
              </div>

              <!-- 工作状态提示：waitingAsk 由 AskUserDialog 抽屉表达；工具调用过渡期由
                AI 回复框内部的空内容兜底（ChatMessageItem 的「正在进行下一步…」）承担 -->
            </div>
          </Transition>
        </div>

        <!-- 目录概览（TOC）：悬浮于消息区右侧，展示各条 AI 回复的摘要标签，点击快捷跳转 -->
        <ToolToc
          :refresh-key="tocRefreshKey"
          container-selector="#experimental-chat-scroll"
          :scroll-offset="16"
          :min-items="2"
        />
      </div>

      <!-- 输入区 -->
      <div class="border-t border-gray-200 bg-white px-4 py-3">
        <div class="mb-1.5 flex items-center gap-1">
          <Button type="text" size="mini" @click="c.attach('crash_report')">
            <template #icon><Squares2X2Icon class="h-3.5 w-3.5" /></template>
            崩溃日志
          </Button>
          <Button type="text" size="mini" @click="c.attach('game_logs')">
            <template #icon><PaperClipIcon class="h-3.5 w-3.5" /></template>
            游戏日志
          </Button>
          <Button type="text" size="mini" @click="c.attach('mods')">
            <template #icon><CommandLineIcon class="h-3.5 w-3.5" /></template>
            Mods 列表
          </Button>
          <span class="flex-1" />
          <!-- 上下文窗口进度条 -->
          <Tooltip :text="`上下文已用 ${c.formatTokens(c.tokenEstimate.value)} / ${c.formatTokens(c.maxInputTokens.value)} token（${c.tokenPercent.value}%）`">
            <div class="flex items-center gap-1.5">
              <div class="relative h-1.5 w-28 overflow-hidden rounded-full bg-gray-200">
                <div
                  class="h-full rounded-full transition-all duration-300"
                  :class="c.tokenBarColor.value"
                  :style="{ width: c.tokenPercent.value + '%' }"
                />
                <div class="progress-sweep" />
              </div>
              <span class="w-[4.5rem] text-right text-[11px] text-gray-400 whitespace-nowrap tabular-nums">
                {{ c.formatTokens(c.tokenEstimate.value) }} / {{ c.formatTokens(c.maxInputTokens.value) }}
              </span>
            </div>
          </Tooltip>
        </div>
        <div class="relative">
          <Input
            v-model="c.inputText.value"
            textarea
            :rows="3"
            class="w-full"
            placeholder="输入消息，Enter 发送，Shift+Enter 换行；如需附带上下文请先点击上方按钮"
            @keydown.enter="c.onEnterKey"
          />
          <Tooltip :text="c.loading.value ? '暂停生成' : '发送'">
            <button
              class="absolute right-2 bottom-2 z-10 rounded-md p-1.5 text-primary-500 transition-colors hover:bg-primary-50 disabled:text-gray-300 disabled:hover:bg-transparent"
              :disabled="!c.inputText.value.trim() && !c.loading.value"
              @click="c.loading.value ? c.cancel() : c.send()"
            >
              <PauseIcon v-if="c.loading.value" class="h-4 w-4" />
              <PaperAirplaneIcon v-else class="h-4 w-4" />
            </button>
          </Tooltip>
        </div>
      </div>
    </section>

    <AskUserDialog
      :visible="c.askUser.value.visible"
      :question="c.askUser.value.question"
      :options="c.askUser.value.options"
      @submit="c.submitAskUser"
      @cancel="c.cancelAskUser"
    />
    <VersionPickerDialog
      :visible="c.versionPicker.value.visible"
      :versions="c.versionPicker.value.versions"
      title="选择游戏版本后收集上下文"
      @select="(v: string) => c.doCollect(c.pendingAttachKind.value, v)"
      @cancel="c.versionPicker.value = { visible: false, versions: [] }"
    />
  </div>
</template>

<style scoped>
/* 输入框底部为右下角发送按钮留出空间 */
:deep(.textarea-inner) {
  padding-bottom: 2rem;
}

/* 会话切换过渡：旧会话淡出上移，新会话淡入 */
.conv-view-enter-active,
.conv-view-leave-active {
  transition:
    opacity 0.18s ease,
    transform 0.18s ease;
}

.conv-view-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.conv-view-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
