<script setup lang="ts">
/**
 * AI 日志深度分析（弹窗）
 *
 * 参考更新日志弹窗（UpdateDialog）的设计语言：
 * - .modal-shell（fixed 遮罩，内部 absolute inset-0 bg-black/40）> .modal-body.max-w-xl.mt-2
 * - 标题栏 px-6 pt-5 pb-3（图标 + text-base 标题 + XMarkIcon 关闭）
 * - 内容区 .modal-scroll px-6 pb-2（限高滚动）
 * - 固定底部按钮栏 bg-gray-50 rounded-b-lg（与 UpdateDialog 一致）
 *
 * 流程：本地引擎转交 AI（externalLogText）→ 自动打开弹窗并发起流式分析。
 * 阶段指示使用 AnalyzeStageBar：思考阶段只显示「深度思考中…」灰底不可用状态，
 * 只有正文输出收到【STEP:N/5】标记时才显示环节进度——思考过程中绝不显示阶段。
 */
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { SparklesIcon, XMarkIcon, ChevronDownIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import AnalyzeStageBar, { type AnalyzeStage } from '@/components/experimental/AnalyzeStageBar.vue'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { aiLoadConfig } from '@/utils/api/ai'
import {
  experimentalManager,
  experimentalAiAnalyzeLog,
  EXPERIMENTAL_ACTIONS,
  type AiAnalyzeStreamEvent,
} from '@/utils/api/experimental'
import { renderMarkdown } from '@/utils/markdown'
import { mountMdIcons } from '@/utils/md-icons'
import { toastError } from '@/utils/toast'

const props = defineProps<{ externalLogText?: string }>()
const emit = defineEmits<{ consumed: [] }>()

const stages: readonly AnalyzeStage[] = [
  { key: 'read', label: '读取整理日志' },
  { key: 'env', label: '环境依赖检查' },
  { key: 'trace', label: '异常链定位' },
  { key: 'root', label: '根因判断' },
  { key: 'fix', label: '修复建议' },
]

const visible = ref(false)
const model = ref('')
const models = ref<string[]>([])
const analyzing = ref(false)
/** 待分析的日志文本（本地引擎初检后传回，点击「用 AI 深度分析」才执行，不默认弹窗） */
const pendingText = ref('')
/** 正文输出当前环节（-1 = 思考阶段，未收到 STEP 标记） */
const currentIndex = ref(-1)
const conclusion = ref('')
const reasoning = ref('')
/** 深度思考日志默认收起，可展开查看 */
const reasoningExpanded = ref(false)
const resultRef = ref<HTMLElement | null>(null)
const reasoningRef = ref<HTMLElement | null>(null)

const modelOptions = computed(() => models.value.map((m) => ({ label: m, value: m })))

const streamEvent = useTauriEvent<AiAnalyzeStreamEvent>('ai-analyze-stream', (ev) => {
  if (!analyzing.value) return
  // 阶段只由正文输出的 STEP 字段驱动：思考过程收到的是 reasoning 增量，绝不会触发阶段显示
  if (typeof ev.step === 'number' && ev.step >= 1 && ev.step <= stages.length) {
    currentIndex.value = ev.step - 1
  }
  if (typeof ev.reasoning === 'string' && ev.reasoning) {
    reasoning.value += ev.reasoning
  }
  if (typeof ev.delta === 'string' && ev.delta) {
    conclusion.value += ev.delta
  }
  if (ev.done) {
    currentIndex.value = stages.length - 1
    if (typeof ev.content === 'string' && ev.content) conclusion.value = ev.content
    analyzing.value = false
  }
  if (typeof ev.error === 'string' && ev.error) {
    analyzing.value = false
    toastError(`AI 分析失败：${ev.error}`)
  }
  if (ev.cancelled) {
    analyzing.value = false
  }
})

// 结论流式更新后，把 v-html 渲染出的 [::名称] 图标占位符挂载为 heroicons 组件
watch(
  conclusion,
  () => {
    if (resultRef.value) mountMdIcons(resultRef.value)
  },
  { flush: 'post' },
)

// 思考日志增量到达且展开时自动滚动到底部
watch(
  reasoning,
  () => {
    if (reasoningExpanded.value && reasoningRef.value) {
      reasoningRef.value.scrollTop = reasoningRef.value.scrollHeight
    }
  },
  { flush: 'post' },
)

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}

function open() {
  visible.value = true
  reasoningExpanded.value = false
  window.addEventListener('keydown', onKey)
}

function close() {
  visible.value = false
  window.removeEventListener('keydown', onKey)
  // 弹窗关闭时若分析仍在进行：通知后端停止 SSE 流
  if (analyzing.value) {
    analyzing.value = false
    void experimentalManager<void>(EXPERIMENTAL_ACTIONS.CANCEL_LOG_ANALYZE).catch(() => {})
  }
}

onMounted(async () => {
  streamEvent.start()
  try {
    const cfg = await aiLoadConfig()
    models.value = cfg.models ?? []
    model.value = cfg.defaultModel || cfg.models?.[0] || ''
  } catch {
    models.value = []
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
})

// 流水线第二步：用户点击「用 AI 深度分析」（CrashAnalyzer 内的入口按钮）后，
// 本地引擎初检日志文本传回 → 打开弹窗，由用户确认模型后再点「开始分析」发起。
// 不自动分析——弹窗内先让用户二次确认模型选择。
watch(
  () => props.externalLogText,
  (text) => {
    if (typeof text !== 'string' || !text.trim()) return
    if (analyzing.value) return
    emit('consumed')
    pendingText.value = text
    open()
  },
  { immediate: false },
)

async function runAnalyze(text?: string) {
  const source = typeof text === 'string' && text.trim() ? text : pendingText.value
  if (!source || !source.trim()) {
    toastError('未获取到日志内容')
    return
  }
  if (!model.value) {
    toastError('请先选择 AI 模型')
    return
  }
  analyzing.value = true
  currentIndex.value = -1
  conclusion.value = ''
  reasoning.value = ''
  reasoningExpanded.value = false
  try {
    await experimentalAiAnalyzeLog({
      logText: source,
      model: model.value,
      reasoningEffort: null,
      // 本地引擎初检已完成 → 后端注入预检范围，避免超长全文直发模型
      localAnalyze: true,
    })
  } catch (e) {
    analyzing.value = false
    toastError(`AI 分析失败: ${e instanceof Error ? e.message : String(e)}`)
  }
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="visible" class="modal-shell" tabindex="0" @click.self="close" @keydown="onKey">
        <div class="absolute inset-0 bg-black/40" />

        <div class="modal-body max-w-xl mt-2">
          <!-- 标题栏 -->
          <div class="flex items-center justify-between px-6 pt-5 pb-3">
            <div class="flex items-center gap-2.5">
              <SparklesIcon class="h-5 w-5 text-primary-500" />
              <h3 class="text-base font-semibold text-gray-900">AI 深度分析</h3>
            </div>
            <button
              class="text-gray-400 hover:text-gray-600 transition-colors"
              @click="close"
            >
              <XMarkIcon class="h-5 w-5" />
            </button>
          </div>

          <!-- 内容区（限高滚动） -->
          <div class="modal-scroll px-6 pb-2">
            <!-- 模型选择 -->
            <div class="flex items-center gap-2 py-2">
              <span class="shrink-0 text-xs text-gray-500">分析模型</span>
              <div class="w-56">
                <Select
                  v-model="model"
                  :options="modelOptions"
                  placeholder="选择模型"
                  size="small"
                  :disabled="analyzing"
                />
              </div>
            </div>
            <p v-if="models.length === 0" class="text-xs text-amber-600">
              未配置 AI 模型，请先在「设置 → AI 设置」中启用并配置模型。
            </p>

            <!-- 阶段指示：思考阶段显示「深度思考中…」（不可用状态），正文输出才显示环节 -->
            <div v-if="analyzing" class="py-1">
              <AnalyzeStageBar :stages="stages" :current-index="currentIndex" />
            </div>

            <!-- 深度思考日志：默认收起，点击展开查看（容器限高内部滚动） -->
            <div
              v-if="reasoning || analyzing"
              class="mt-2 overflow-hidden rounded-md border border-gray-100"
            >
              <button
                type="button"
                class="flex w-full items-center gap-1.5 px-3 py-2 text-xs font-medium text-gray-500 transition-colors hover:bg-gray-50"
                @click="reasoningExpanded = !reasoningExpanded"
              >
                <span
                  v-if="analyzing"
                  class="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-primary-500"
                />
                <span class="flex-1 text-left">思考过程</span>
                <ChevronDownIcon
                  class="h-3.5 w-3.5 transition-transform"
                  :class="reasoningExpanded ? 'rotate-180' : ''"
                />
              </button>
              <div
                v-if="reasoningExpanded && reasoning"
                ref="reasoningRef"
                class="max-h-60 overflow-auto whitespace-pre-wrap border-t border-gray-100 px-3 py-2.5 text-xs leading-relaxed text-gray-500"
              >
                {{ reasoning }}
              </div>
            </div>

            <!-- 结论（Markdown，流式） -->
            <div v-if="conclusion" class="pt-2">
              <div class="mb-1.5 text-xs font-medium text-gray-500">分析结论</div>
              <div
                class="max-h-72 overflow-auto rounded-md bg-gray-50"
              >
                <div
                  ref="resultRef"
                  class="markdown-body px-3 py-2.5 text-sm leading-relaxed text-gray-700"
                  v-html="renderMarkdown(conclusion)"
                />
              </div>
            </div>

            <!-- 空状态（弹窗刚打开，等待用户确认模型并点击「开始分析」） -->
            <div
              v-if="!analyzing && !conclusion"
              class="flex flex-col items-center justify-center py-10 text-gray-400"
            >
              <SparklesIcon class="mb-2 h-8 w-8" />
              <span class="text-xs">确认模型后点击「开始分析」启动 AI 深度分析</span>
            </div>
          </div>

          <!-- 底部按钮栏（与 UpdateDialog 一致） -->
          <div class="flex justify-end gap-2 rounded-b-lg bg-gray-50 px-6 py-3.5">
            <Button v-if="conclusion && !analyzing" type="ghost" size="small" @click="close">关闭</Button>
            <Button type="primary" size="small" :loading="analyzing" :disabled="analyzing || !model" @click="runAnalyze()">
              <template #icon><SparklesIcon class="h-4 w-4" /></template>
              {{ analyzing ? '分析中' : conclusion ? '重新分析' : '开始分析' }}
            </Button>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
