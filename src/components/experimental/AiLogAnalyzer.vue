<script setup lang="ts">
/**
 * AI 日志分析
 *
 * 选择 AI 模型并粘贴日志文本，调用 experimentalAiAnalyzeLog 发起流式分析。
 * 后端通过 ai-analyze-stream 事件推送环节进度（【STEP:N/5】）与结论增量（delta），
 * 前端用 StepProgressBar（扫光）驱动 5 环节进度条，并流式渲染 Markdown 结论。
 */
import { ref, computed, watch, onMounted } from 'vue'
import { MagnifyingGlassIcon, SparklesIcon, DocumentTextIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import StepProgressBar from '@/components/common/StepProgressBar.vue'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { aiLoadConfig } from '@/utils/api/ai'
import { experimentalAiAnalyzeLog, type AiAnalyzeStreamEvent } from '@/utils/api/experimental'
import { renderMarkdown } from '@/utils/markdown'
import { mountMdIcons } from '@/utils/md-icons'
import { toastError } from '@/utils/toast'

/**
 * 流水线第二步：接收本地引擎初检后传回的日志文本，AI 深度分析（后端 localAnalyze=true
 * 会把本地预检范围注入上下文，避免超长全文直发模型）
 */
const props = defineProps<{ externalLogText?: string }>()
const emit = defineEmits<{ consumed: [] }>()

const steps = [
  { key: 'read', label: '读取整理日志' },
  { key: 'env', label: '环境依赖检查' },
  { key: 'trace', label: '异常链定位' },
  { key: 'root', label: '根因判断' },
  { key: 'fix', label: '修复建议' },
]

const logText = ref('')
const model = ref('')
const models = ref<string[]>([])
const analyzing = ref(false)
const currentIndex = ref(-1)
const conclusion = ref('')
const resultRef = ref<HTMLElement | null>(null)

const canAnalyze = computed(() => logText.value.trim().length > 0 && model.value !== '')
const modelOptions = computed(() => models.value.map((m) => ({ label: m, value: m })))

const streamEvent = useTauriEvent<AiAnalyzeStreamEvent>('ai-analyze-stream', (ev) => {
  if (!analyzing.value) return
  if (typeof ev.step === 'number' && ev.step >= 1 && ev.step <= steps.length) {
    currentIndex.value = ev.step - 1
  }
  if (typeof ev.delta === 'string' && ev.delta) {
    conclusion.value += ev.delta
  }
  if (ev.done) {
    currentIndex.value = steps.length - 1
    if (typeof ev.content === 'string' && ev.content) conclusion.value = ev.content
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

// 流水线第二步：本地引擎初检完成后把日志文本传回，自动填充并触发 AI 深度分析
watch(
  () => props.externalLogText,
  (text) => {
    if (typeof text !== 'string' || !text.trim()) return
    if (analyzing.value) return
    logText.value = text
    emit('consumed')
    void runAnalyze()
  },
  { immediate: false },
)

async function runAnalyze() {
  if (!logText.value.trim()) {
    toastError('请先粘贴日志内容')
    return
  }
  if (!model.value) {
    toastError('请先选择 AI 模型')
    return
  }
  analyzing.value = true
  currentIndex.value = -1
  conclusion.value = ''
  try {
    await experimentalAiAnalyzeLog({
      logText: logText.value,
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

function clearAll() {
  logText.value = ''
  conclusion.value = ''
  currentIndex.value = -1
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <SparklesIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">AI 日志分析</h3>
    </div>
    <div class="space-y-3 px-5 pb-5">
      <p class="text-xs text-gray-500">
        选择 AI 模型并粘贴日志文本，AI 将按「读取整理日志 → 环境依赖检查 → 异常链定位 → 根因判断 → 修复建议」五个环节流式输出诊断结论。
      </p>

      <!-- 模型选择 -->
      <div class="flex items-center gap-2">
        <span class="shrink-0 text-xs font-medium text-gray-600">分析模型</span>
        <div class="w-64">
          <Select v-model="model" :options="modelOptions" placeholder="选择模型" :disabled="analyzing" />
        </div>
      </div>
      <p v-if="models.length === 0" class="text-xs text-amber-600">
        未配置 AI 模型，请先在「设置 → AI 设置」中启用并配置模型。
      </p>

      <!-- 日志输入 -->
      <Input
        v-model="logText"
        textarea
        :rows="8"
        :disabled="analyzing"
        placeholder="在此粘贴日志文本（crash report 或 latest.log 中的报错片段）..."
      />

      <!-- 操作按钮 -->
      <div class="flex justify-end gap-2">
        <Button type="ghost" size="small" :disabled="analyzing || !logText" @click="clearAll">清空</Button>
        <Button type="primary" :loading="analyzing" :disabled="!canAnalyze" @click="runAnalyze">
          <template #icon><MagnifyingGlassIcon class="h-4 w-4" /></template>
          {{ analyzing ? '分析中...' : '开始分析' }}
        </Button>
      </div>

      <!-- 分析过程：环节进度条（扫光） -->
      <div v-if="analyzing" class="rounded-lg border border-gray-200 px-4 py-3">
        <StepProgressBar :steps="steps" :current-index="currentIndex" sweep />
      </div>

      <!-- 结论（Markdown） -->
      <div v-else-if="conclusion" class="rounded-lg border border-gray-200 px-4 py-3">
        <div class="mb-2 text-sm font-medium text-gray-700">分析结论</div>
        <div
          ref="resultRef"
          class="markdown-body text-sm leading-relaxed text-gray-700"
          v-html="renderMarkdown(conclusion)"
        />
      </div>

      <!-- 空状态：icon + text 垂直水平居中 -->
      <div v-else class="flex flex-col items-center justify-center py-10 text-gray-400">
        <DocumentTextIcon class="mb-2 h-8 w-8" />
        <span class="text-xs">粘贴日志并点击「开始分析」，AI 将分环节输出诊断结论</span>
      </div>
    </div>
  </section>
</template>