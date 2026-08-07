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
 */
import { ref, watch, onBeforeUnmount } from 'vue'
import { SparklesIcon, XMarkIcon, ChevronDownIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import AnalyzeStageBar, { type AnalyzeStage } from '@/components/experimental/AnalyzeStageBar.vue'
import { useAiLogAnalyzer } from '@/composables/useAiLogAnalyzer'
import { renderMarkdown } from '@/utils/markdown'
import { mountMdIcons } from '@/utils/md-icons'

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
const pendingText = ref('')
const reasoningExpanded = ref(false)
const resultRef = ref<HTMLElement | null>(null)
const reasoningRef = ref<HTMLElement | null>(null)
const analysis = useAiLogAnalyzer(stages.length)
const {
  model,
  models,
  analyzing,
  currentIndex,
  conclusion,
  reasoning,
  modelOptions,
  runAnalyze: startAnalyze,
  cancel,
} = analysis

watch(conclusion, () => {
  if (resultRef.value) mountMdIcons(resultRef.value)
}, { flush: 'post' })

watch(reasoning, () => {
  if (reasoningExpanded.value && reasoningRef.value) {
    reasoningRef.value.scrollTop = reasoningRef.value.scrollHeight
  }
}, { flush: 'post' })

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
  cancel()
}

onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

watch(
  () => props.externalLogText,
  (text) => {
    if (typeof text !== 'string' || !text.trim() || analyzing.value) return
    emit('consumed')
    pendingText.value = text
    open()
  },
)

async function runAnalyze(text?: string) {
  const source = typeof text === 'string' && text.trim() ? text : pendingText.value
  reasoningExpanded.value = false
  await startAnalyze(source)
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
          <div class="flex items-center justify-between px-6 pt-5 pb-3">
            <div class="flex items-center gap-2.5">
              <SparklesIcon class="h-5 w-5 text-primary-500" />
              <h3 class="text-base font-semibold text-gray-900">AI 深度分析</h3>
            </div>
            <button class="text-gray-400 hover:text-gray-600 transition-colors" @click="close">
              <XMarkIcon class="h-5 w-5" />
            </button>
          </div>

          <div class="modal-scroll px-6 pb-2">
            <div class="flex items-center gap-2 py-2">
              <span class="shrink-0 text-xs text-gray-500">分析模型</span>
              <div class="w-56">
                <Select v-model="model" :options="modelOptions" placeholder="选择模型" size="small" :disabled="analyzing" />
              </div>
            </div>
            <p v-if="models.length === 0" class="text-xs text-amber-600">
              未配置 AI 模型，请先在「设置 → AI 设置」中启用并配置模型。
            </p>

            <div v-if="analyzing" class="py-1">
              <AnalyzeStageBar :stages="stages" :current-index="currentIndex" />
            </div>

            <div v-if="reasoning || analyzing" class="mt-2 overflow-hidden rounded-md border border-gray-100">
              <button
                type="button"
                class="flex w-full items-center gap-1.5 px-3 py-2 text-xs font-medium text-gray-500 transition-colors hover:bg-gray-50"
                @click="reasoningExpanded = !reasoningExpanded"
              >
                <span v-if="analyzing" class="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-primary-500" />
                <span class="flex-1 text-left">思考过程</span>
                <ChevronDownIcon class="h-3.5 w-3.5 transition-transform" :class="reasoningExpanded ? 'rotate-180' : ''" />
              </button>
              <div v-if="reasoningExpanded && reasoning" ref="reasoningRef" class="max-h-60 overflow-auto whitespace-pre-wrap border-t border-gray-100 px-3 py-2.5 text-xs leading-relaxed text-gray-500">
                {{ reasoning }}
              </div>
            </div>

            <div v-if="conclusion" class="pt-2">
              <div class="mb-1.5 text-xs font-medium text-gray-500">分析结论</div>
              <div class="max-h-72 overflow-auto rounded-md bg-gray-50">
                <div ref="resultRef" class="markdown-body px-3 py-2.5 text-sm leading-relaxed text-gray-700" v-html="renderMarkdown(conclusion)" />
              </div>
            </div>

            <div v-if="!analyzing && !conclusion" class="flex flex-col items-center justify-center py-10 text-gray-400">
              <SparklesIcon class="mb-2 h-8 w-8" />
              <span class="text-xs">确认模型后点击「开始分析」启动 AI 深度分析</span>
            </div>
          </div>

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
