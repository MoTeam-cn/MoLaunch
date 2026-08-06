<script setup lang="ts">
/**
 * 单条聊天消息
 * Markdown 渲染、流式光标、hover 操作栏（时间 / 删除 / 重新生成 / 复制 / 编辑）。
 */
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  ArrowPathIcon,
  ChevronDownIcon,
  ChevronUpIcon,
  ClipboardIcon,
  PencilIcon,
  TrashIcon,
} from '@heroicons/vue/24/outline'
import ModelIcon from '@/components/common/ModelIcon.vue'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import CopyMessageDialog from '@/components/experimental/CopyMessageDialog.vue'
import { renderMarkdown, handleMarkdownLinkClick } from '@/utils/markdown'
import { mountMdIcons } from '@/utils/md-icons'
import { formatTimestamp } from '@/utils/format'
import { formatTokens } from '@/utils/tokens'
import type { MessageItem } from '@/utils/api/experimental'

export type LocalMessage = MessageItem & { streaming?: boolean }

const props = defineProps<{
  message: LocalMessage
  isLastUser?: boolean
  model?: string | null
  busy?: boolean
  /** 流式占位消息实时展示的生成速度（t/s，输出 token ÷ 总耗时，来自 done 事件） */
  liveSpeed?: number
  /** 流式占位消息实时展示的输出 token（completion，来自 done 事件） */
  liveCompletion?: number
}>()

/** 回复该消息的模型：优先消息自身记录（切换全局模型后仍固定），否则用当前模型兜底 */
const displayModel = computed(() => props.message.model || props.model || null)
/** 消息发出/回复时间（Unix 秒） */
const timeText = computed(() => formatTimestamp(props.message.createdAt, { withSeconds: false }))
/** 「第 N 次重试」标识：生成序号 > 1 时显示（首次为 1 不显示） */
const retryLabel = computed(() => {
  const n = props.message.retryCount
  if (!n || n <= 1) return ''
  return `第 ${n - 1} 次重试`
})
/**
 * 本次回复 token 统计：显示输出 token（completion，含全部工具轮次累计）；
 * 速度 = 输出 token ÷ 总耗时。优先历史持久化的真实 usage，流式占位用 done 事件实时值
 */
const tokenStats = computed(() => {
  const completion =
    props.message.completionTokens ?? (props.message.streaming ? props.liveCompletion : null)
  if (!completion || completion <= 0) return null
  let speed = 0
  if (props.message.durationMs && props.message.durationMs > 0) {
    speed = Math.round((completion * 1000) / props.message.durationMs)
  } else if (props.message.streaming) {
    speed = props.liveSpeed ?? 0
  }
  return { completion, speed }
})

const emit = defineEmits<{
  delete: []
  regenerate: []
  edit: [content: string]
}>()

const copyOpen = ref(false)
const editing = ref(false)
const editText = ref('')
/** markdown 正文容器：渲染后把 `[::名称]` 图标占位符替换为 heroicons 组件 */
const markdownBodyRef = ref<HTMLElement | null>(null)
/** 监听 markdown 容器 DOM 变化（初始渲染 / 流式追加 / 消息替换都会触发），
 * 把新增的 `.md-icon` 占位符挂载为 heroicons 组件；已挂载的跳过。
 * 流式占位消息挂载时 content 为空、容器尚未渲染，需在 content 首次非空后再绑定 */
let iconObserver: MutationObserver | null = null
function attachIconObserver() {
  if (!markdownBodyRef.value || iconObserver) return
  mountMdIcons(markdownBodyRef.value)
  iconObserver = new MutationObserver(() => {
    if (markdownBodyRef.value) mountMdIcons(markdownBodyRef.value)
  })
  iconObserver.observe(markdownBodyRef.value, { childList: true, subtree: true })
}
onMounted(() => attachIconObserver())
watch(
  () => props.message.content,
  () => attachIconObserver(),
  { flush: 'post' },
)
onBeforeUnmount(() => iconObserver?.disconnect())
/** 「深度思考」折叠：流式生成时自动展开，生成完成后自动折叠 */
const thinkingOpen = ref(false)
watch([() => props.message.reasoningContent, () => props.message.streaming], ([v, s]) => {
  if (s && v) thinkingOpen.value = true
  else if (!s) thinkingOpen.value = false
})

function startEdit() {
  editText.value = props.message.content
  editing.value = true
  copyOpen.value = false
}

function cancelEdit() {
  editing.value = false
}

function saveEdit() {
  const text = editText.value.trim()
  if (!text) return
  editing.value = false
  emit('edit', text)
}
</script>

<template>
  <div class="group relative">
    <!-- 用户消息 -->
    <div v-if="message.role === 'user'" class="flex justify-end">
      <div
        v-if="editing"
        class="w-full max-w-[88%] space-y-2 rounded-lg border border-primary-300 bg-white px-3 py-2"
      >
        <Input v-model="editText" textarea :rows="4" class="w-full" @keydown.enter.exact.prevent="saveEdit" />
        <div class="flex justify-end gap-1.5">
          <Button type="ghost" size="mini" @click="cancelEdit">取消</Button>
          <Button type="primary" size="mini" :disabled="!editText.trim()" @click="saveEdit">保存并重新生成</Button>
        </div>
      </div>
      <div
        v-else
        class="max-w-[78%] rounded-lg bg-primary-500 px-4 py-2.5 text-sm text-white whitespace-pre-wrap break-words"
      >{{ message.content }}</div>
    </div>

    <!-- AI 消息 -->
    <div v-else class="flex justify-start">
      <ModelIcon :model="displayModel" class="mt-1 mr-2 h-6 w-6 shrink-0 text-gray-400" />
      <div class="min-w-0 max-w-[82%] rounded-lg border border-gray-200 bg-gray-50 px-4 py-2.5 text-sm text-gray-800">
        <!-- 顶部元信息行：深度思考切换（左）+ 重试次数标识（右，ml-auto 靠右）同行展示 -->
        <div v-if="message.reasoningContent || retryLabel" class="mb-1.5 flex items-center gap-2">
          <button
            v-if="message.reasoningContent"
            type="button"
            class="flex items-center gap-1 text-xs font-medium text-gray-500 hover:text-gray-700"
            @click="thinkingOpen = !thinkingOpen"
          >
            <ChevronDownIcon v-if="!thinkingOpen" class="h-3.5 w-3.5" />
            <ChevronUpIcon v-else class="h-3.5 w-3.5" />
            <span>深度思考</span>
          </button>
          <span
            v-if="retryLabel"
            class="ml-auto rounded bg-amber-100 px-1.5 py-0.5 text-[10px] leading-none whitespace-nowrap text-amber-700"
          >{{ retryLabel }}</span>
        </div>
        <!-- 深度思考内容（仅展开时显示；虚线分隔思考内容与正文，收起时不渲染避免撑出多余空隙） -->
        <div v-if="message.reasoningContent && thinkingOpen" class="mb-1.5 border-b border-dashed border-gray-200 pb-1.5">
          <div
            class="max-h-64 overflow-y-auto rounded bg-gray-100/90 px-2.5 py-2 text-xs leading-relaxed whitespace-pre-wrap break-words text-gray-600"
          >{{ message.reasoningContent }}</div>
        </div>
        <div
          v-if="message.content"
          ref="markdownBodyRef"
          class="markdown-body"
          @click="handleMarkdownLinkClick"
          v-html="renderMarkdown(message.content)"
        />
        <div v-else-if="!message.reasoningContent" class="flex items-center gap-1.5 text-gray-400">
          <span class="stream-caret" />
          <span>正在思考</span>
        </div>
        <span v-if="message.streaming && message.content" class="stream-caret text-primary-500" />
        <!-- 右下角：token 统计（模型 id 左侧）+ 回复该消息的模型（仅最终正文存在时显示） -->
        <div v-if="(displayModel && message.content) || tokenStats" class="mt-1.5 flex items-center justify-end gap-1.5">
          <span v-if="tokenStats" class="rounded bg-gray-200/90 px-1.5 py-0.5 text-[10px] leading-none whitespace-nowrap text-gray-500 tabular-nums">
            {{ formatTokens(tokenStats.completion) }} token<span v-if="tokenStats.speed > 0"> · {{ formatTokens(tokenStats.speed) }} t/s</span>
          </span>
          <span v-if="displayModel && message.content" class="rounded bg-gray-200/90 px-1.5 py-0.5 text-[10px] leading-none text-gray-500">{{ displayModel }}</span>
        </div>
      </div>
    </div>
    <!-- 操作栏（hover / 聚焦时显示：消息时间 + 功能按钮） -->
    <div
      class="flex px-1 py-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100 group-focus-within:opacity-100"
      :class="message.role === 'user' ? 'justify-end' : 'ml-8 justify-start'"
    >
      <!-- 时间：用户消息在按钮左侧；AI 消息在按钮右侧（order-last） -->
      <span
        class="self-center text-[11px] text-gray-400"
        :class="message.role === 'user' ? 'mr-2' : 'order-last ml-2'"
      >{{ timeText }}</span>
      <div class="relative flex items-center gap-0.5">
        <Tooltip text="删除消息">
          <button
            class="rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-red-500 disabled:opacity-40"
            :disabled="busy || message.id === 0"
            @click="emit('delete')"
          >
            <TrashIcon class="h-3.5 w-3.5" />
          </button>
        </Tooltip>
        <Tooltip v-if="message.role === 'assistant'" text="重新生成">
          <button
            class="rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-primary-500 disabled:opacity-40"
            :disabled="busy || message.id === 0"
            @click="emit('regenerate')"
          >
            <ArrowPathIcon class="h-3.5 w-3.5" />
          </button>
        </Tooltip>
        <Tooltip text="复制消息">
          <button
            class="rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-primary-500 disabled:opacity-40"
            :disabled="busy"
            @click="copyOpen = true"
          >
            <ClipboardIcon class="h-3.5 w-3.5" />
          </button>
        </Tooltip>
        <Tooltip v-if="message.role === 'user' && isLastUser && !message.streaming" text="编辑消息">
          <button
            class="rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-primary-500 disabled:opacity-40"
            :disabled="busy || message.id === 0"
            @click="startEdit"
          >
            <PencilIcon class="h-3.5 w-3.5" />
          </button>
        </Tooltip>
      </div>
    </div>

    <!-- 复制消息弹窗（预览 + 选择复制格式，点击外部/ESC 关闭） -->
    <CopyMessageDialog v-model="copyOpen" :content="message.content" />
  </div>
</template>

<style scoped>
/* 流式生成光标 */
.stream-caret {
  display: inline-block;
  width: 2px;
  height: 1em;
  vertical-align: -0.125em;
  background: currentColor;
  animation: stream-blink 1s steps(1, end) infinite;
}
@keyframes stream-blink {
  50% {
    opacity: 0;
  }
}
</style>
