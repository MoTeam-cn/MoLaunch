<script setup lang="ts">
/**
 * 模组翻译 - 分析结果区（左右分栏）：左侧内容区 + 右侧操作区。
 * 设置项经 defineModel 双向绑定，操作经 emit 上报父组件。
 */
import { ref, computed, watch, onUnmounted, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
const Collapse = defineAsyncComponent(() => import('@/components/common/Collapse.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import * as tauri from '@/utils/tauri'
import { toastError } from '@/utils/toast'
import type {
  ModTranslationAnalyzeResult,
  ModTranslationTaskSnapshot,
} from '@/utils/api/experimental-mod-translation'

const props = defineProps<{
  analyzeResult: ModTranslationAnalyzeResult
  snapshot: ModTranslationTaskSnapshot | null
  running: boolean
  completed: boolean
  taskFakeProgress: number
  modelOptions: { label: string; value: string }[]
}>()
const model = defineModel<string>('model', { default: '' })
const batchSize = defineModel<number>('batch-size', { default: 40 })
const generateModName = defineModel<boolean>('generate-mod-name', { default: true })
const repairEnabled = defineModel<boolean>('repair-enabled', { default: true })
const classTextEnabled = defineModel<boolean>('class-text-enabled', { default: true })
const emit = defineEmits<{ start: []; cancel: []; back: [] }>()

const batchOptions = [
  { label: '20 条/批（更稳）', value: 20 },
  { label: '40 条/批（推荐）', value: 40 },
  { label: '80 条/批（更快）', value: 80 },
]
/** i18n 翻译模组/资源包（MC百科） */
const MCMOD_I18N_URL = 'https://www.mcmod.cn/class/1188.html'
const detailOpen = ref(false)

const loaderLabels: Record<string, string> = {
  fabric: 'Fabric',
  neoforge: 'NeoForge',
  forge: 'Forge',
  unknown: '未知',
}
const kindLabels: Record<string, string> = {
  json: 'JSON 语言文件',
  'key-value': '.lang/.properties',
  'structured-json': '结构化 JSON',
  'free-text': '自由文本',
}
const dispositionLabels: Record<string, string> = {
  standard_language: '标准语言',
  structured_source: '结构化源',
  generated_target: '生成目标',
  class_review: 'class 复核',
  unknown: '未知',
  protected: '保护',
}
const modNameSourceLabels: Record<string, string> = {
  embedded_chinese: '内嵌中文',
  ai_recommended: 'AI 推荐',
  known_chinese: '已知译名',
  translated_display_name: '直译显示名',
  translated_filename: '直译文件名',
  original_filename: '原文件名',
  display_name: '显示名',
  mod_id: 'Mod ID',
}

const statusText = computed(() => {
  const s = props.snapshot
  if (!s) return ''
  if (s.status === 'completed') return s.message || '翻译完成'
  if (s.status === 'failed') return s.error || '任务失败'
  if (s.status === 'cancelled') return '任务已取消'
  return s.message || '翻译中...'
})
const failed = computed(() => props.snapshot?.status === 'failed')
/** 阶段文字（后端 stage 映射，未知阶段回退原文） */
const stageLabels: Record<string, string> = {
  analyze: '分析中',
  language: '语言翻译',
  repair: '质量复验',
  class: 'class 文本',
  validation: '校验中',
  package: '打包中',
  translate: '翻译中',
}
/** 当前阶段动画点（1-5 循环，running 期间每 300ms 递增） */
const dotCount = ref(0)
let dotTimer: number | null = null
watch(
  () => props.running,
  (running) => {
    if (running) {
      dotTimer = window.setInterval(() => {
        dotCount.value = (dotCount.value % 5) + 1
      }, 300)
    } else {
      if (dotTimer !== null) {
        window.clearInterval(dotTimer)
        dotTimer = null
      }
      dotCount.value = 0
    }
  },
)
onUnmounted(() => {
  if (dotTimer !== null) window.clearInterval(dotTimer)
})
/** 展示总进度：后端按阶段权重加权计算，running 期间与假进度取较大值 */
const displayProgress = computed(() => {
  if (!props.snapshot) return 0
  return props.running ? Math.max(props.snapshot.progress, props.taskFakeProgress) : props.snapshot.progress
})
/** 分进度折叠区开关（默认折叠） */
const stageOpen = ref(false)

async function handleOpenDir() {
  if (!props.snapshot?.outputPath) return
  try {
    await tauri.openPath(props.snapshot.outputPath.replace(/\\[^\\/]+$/, ''))
  } catch (e) {
    toastError('打开目录失败：' + e)
  }
}
</script>

<template>
  <div class="flex h-full flex-col gap-4 overflow-hidden">
    <!-- 顶部：返回上传 + 当前文件 -->
    <div class="flex items-center gap-3 shrink-0">
      <Button type="ghost" size="small" @click="emit('back')">
        <span class="flex items-center gap-1">← 返回上传</span>
      </Button>
      <Tooltip :text="props.analyzeResult.filename" overflow-only class="flex-1 min-w-0">
        <span class="block truncate text-sm text-gray-700">{{ props.analyzeResult.filename }}</span>
      </Tooltip>
    </div>

    <!-- 功能定位提示 -->
    <Alert variant="soft" type="warning" class="shrink-0" :link="{ url: MCMOD_I18N_URL }">
      <template #default="{ openLink }">
        <p>
          本模组翻译功能仍处于调整优化阶段，受 AI 大模型幻觉影响，翻译准确率与可用性可能不尽如人意，我们会持续优化提示词。建议优先使用更稳定、更完善的
          <a class="text-amber-600 underline cursor-pointer" @click="openLink">i18n 翻译模组或资源包</a>；
          本功能仅作为其尚未翻译或翻译较少的模组的备选方案。
        </p>
      </template>
    </Alert>

    <!-- 左右分栏 -->
    <div class="flex flex-1 min-h-0 gap-4">
      <!-- 左侧：分析结果（内容区，超高内部滚动） -->
      <div class="flex-1 min-w-0 flex flex-col bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 pt-4 pb-3 border-b border-gray-100 shrink-0">
          <h3 class="text-sm font-semibold text-gray-900">2. 分析结果</h3>
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto px-5 py-3">
          <div class="space-y-2 text-sm">
            <div class="flex items-center gap-2">
              <span class="text-gray-500 w-16 shrink-0">加载器</span>
              <span class="text-gray-800">{{ loaderLabels[props.analyzeResult.loader] ?? props.analyzeResult.loader }}</span>
            </div>
            <div v-if="props.analyzeResult.modIds.length" class="flex items-center gap-2">
              <span class="text-gray-500 w-16 shrink-0">Mod ID</span>
              <span class="text-gray-800">{{ props.analyzeResult.modIds.join(', ') }}</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-gray-500 w-16 shrink-0">条目数</span>
              <span class="text-gray-800">{{ props.analyzeResult.totalEntries }}</span>
            </div>
            <div v-if="props.analyzeResult.version" class="flex items-center gap-2">
              <span class="text-gray-500 w-16 shrink-0">版本</span>
              <span class="text-gray-800">{{ props.analyzeResult.version }}</span>
            </div>
            <div v-if="props.analyzeResult.classCandidates.length" class="flex items-center gap-2">
              <span class="text-gray-500 w-16 shrink-0">class 文本</span>
              <span class="text-gray-800">{{ props.analyzeResult.classCandidates.length }} 个候选</span>
            </div>
            <div v-if="props.analyzeResult.existingChinese.length" class="mt-3">
              <Alert variant="soft" type="warning">
                <p>该模组已包含 {{ props.analyzeResult.existingChinese.length }} 个中文语言文件，翻译将覆盖以下文件：</p>
                <ul class="mt-1 space-y-0.5">
                  <li
                    v-for="item in props.analyzeResult.existingChinese"
                    :key="item.path"
                    class="text-yellow-700"
                  >
                    <Tooltip :text="`${item.locale} · ${item.path}（${item.entries} 条）`" overflow-only>
                      <span class="block truncate">{{ item.locale }} · {{ item.path }}（{{ item.entries }} 条）</span>
                    </Tooltip>
                  </li>
                </ul>
              </Alert>
            </div>
            <div v-if="props.analyzeResult.signed" class="flex items-center gap-2">
              <span class="text-yellow-600">JAR 含签名文件，重打包后签名将失效</span>
            </div>
            <div v-for="warn in props.analyzeResult.warnings" :key="warn" class="flex items-center gap-2">
              <span class="text-yellow-600">{{ warn }}</span>
            </div>
          </div>

          <div class="mt-4 border border-gray-200 rounded overflow-hidden">
            <div class="bg-gray-50 px-3 py-2 text-xs text-gray-500 flex items-center gap-3 border-b border-gray-200">
              <span class="w-32 shrink-0">类型</span>
              <span class="flex-1 truncate">文件</span>
              <span class="w-16 text-right shrink-0">待译条目</span>
            </div>
            <div class="max-h-48 overflow-y-auto divide-y divide-gray-100">
              <div
                v-for="source in props.analyzeResult.sources"
                :key="source.sourcePath"
                class="px-3 py-2 text-xs flex items-center gap-3"
              >
                <span class="w-32 shrink-0 text-gray-500">{{ kindLabels[source.kind] ?? source.kind }}</span>
                <Tooltip :text="source.targetPath" overflow-only class="flex-1 min-w-0">
                  <span class="block truncate text-gray-700">{{ source.targetPath }}</span>
                </Tooltip>
                <span class="w-16 text-right shrink-0 text-gray-700">{{ source.entries }}</span>
              </div>
            </div>
          </div>

          <div class="mt-4">
            <button
              class="flex items-center gap-1 text-xs text-gray-500 hover:text-gray-700"
              @click="detailOpen = !detailOpen"
            >
              <svg
                class="w-3 h-3 transition-transform duration-200"
                :class="detailOpen ? 'rotate-90' : ''"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
              </svg>
              成本与覆盖分析
            </button>
            <Collapse :open="detailOpen">
              <div class="mt-2 space-y-3 rounded-lg border border-dashed border-gray-300 p-3">
                <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-gray-600">
                  <div>预估 token：{{ props.analyzeResult.quote.estimatedTokens }}</div>
                  <div>调用次数：{{ props.analyzeResult.quote.estimatedCalls }}</div>
                  <div>语言批次：{{ props.analyzeResult.quote.languageBatches }}</div>
                  <div>class 批次：{{ props.analyzeResult.quote.classBatches }}</div>
                  <div>预估点数：{{ props.analyzeResult.quote.points }}</div>
                </div>
                <div class="border border-gray-200 rounded overflow-hidden">
                  <div class="max-h-40 overflow-y-auto divide-y divide-gray-100">
                    <div
                      v-for="item in props.analyzeResult.coverage"
                      :key="item.path"
                      class="px-3 py-1.5 text-xs flex items-center gap-2"
                    >
                      <Tooltip :text="item.path" overflow-only class="flex-1 min-w-0">
                        <span class="block truncate text-gray-700">{{ item.path }}</span>
                      </Tooltip>
                      <Tag size="small" color="gray">{{ dispositionLabels[item.disposition] ?? item.disposition }}</Tag>
                    </div>
                  </div>
                </div>
              </div>
            </Collapse>
          </div>
        </div>
      </div>

      <!-- 右侧：操作区（固定宽度，超高内部滚动） -->
      <div class="w-80 shrink-0 min-h-0 flex flex-col gap-4 overflow-y-auto">
        <!-- 翻译设置 -->
        <div v-if="!props.running" class="bg-white rounded-lg border border-gray-300 p-5">
          <h3 class="text-sm font-semibold text-gray-900 mb-3">3. 翻译设置</h3>
          <div class="space-y-3">
            <div class="flex items-center gap-3">
              <span class="text-sm text-gray-500 w-16 shrink-0">模型</span>
              <Select v-model="model" :options="props.modelOptions" placeholder="选择翻译模型" />
            </div>
            <div class="flex items-center gap-3">
              <span class="text-sm text-gray-500 w-16 shrink-0">批次</span>
              <Select v-model="batchSize" :options="batchOptions" />
            </div>
            <div class="flex items-center gap-3">
              <span class="text-sm text-gray-500 w-16 shrink-0">选项</span>
              <div class="flex flex-col gap-2">
                <Checkbox v-model="generateModName">生成中文名</Checkbox>
                <Checkbox v-model="repairEnabled">质量回修</Checkbox>
                <Checkbox v-model="classTextEnabled">class 文本</Checkbox>
              </div>
            </div>
            <div class="pt-2">
              <Button type="primary" class="w-full" @click="emit('start')">开始翻译</Button>
            </div>
          </div>
        </div>

        <!-- 任务进度 -->
        <div v-if="props.snapshot && props.snapshot.status !== 'idle'" class="bg-white rounded-lg border border-gray-300 p-5">
          <div class="flex items-center justify-between gap-2 mb-2">
            <span class="text-sm font-semibold" :class="failed ? 'text-red-600' : 'text-gray-900'">
              {{ statusText }}<span v-if="props.running" class="text-primary-500">{{ '.'.repeat(dotCount) }}</span>
            </span>
            <div class="flex items-center gap-2">
              <span v-if="props.running" class="text-xs text-gray-500">{{ Math.round(displayProgress) }}%</span>
              <Button v-if="props.running" type="ghost" size="small" @click="emit('cancel')">取消</Button>
            </div>
          </div>
          <!-- 总进度条（按阶段权重加权计算） -->
          <div class="h-1.5 w-full overflow-hidden rounded-full bg-gray-200">
            <div
              class="h-full bg-primary-500 transition-all duration-200"
              :style="{ width: displayProgress + '%' }"
            />
          </div>
          <!-- AI 异常兜底提示 -->
          <div class="mt-1.5 text-xs text-gray-400">
            AI 偶发异常（如返回空结果）会自动跳过对应条目，不影响整体翻译
          </div>
          <!-- 总进度行：折叠开关 + 重试信息 -->
          <div class="mt-1.5 flex items-center justify-between text-xs text-gray-500">
            <span>总进度 {{ Math.round(displayProgress) }}%</span>
            <div class="flex items-center gap-2">
              <span v-if="props.running && props.snapshot.retry" class="text-yellow-600">
                第 {{ props.snapshot.retry.attempt }}/{{ props.snapshot.retry.total }} 次重试
              </span>
              <button
                v-if="props.snapshot.stages.length"
                class="flex items-center gap-0.5 hover:text-gray-700"
                @click="stageOpen = !stageOpen"
              >
                <svg
                  class="w-3 h-3 transition-transform duration-200"
                  :class="stageOpen ? 'rotate-90' : ''"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
                </svg>
                各阶段
              </button>
            </div>
          </div>
          <!-- 分进度折叠区：所有阶段进度（未开始 0% + 暂未开始） -->
          <Collapse :open="stageOpen">
            <div class="mt-2 space-y-2">
              <div
                v-for="s in props.snapshot.stages"
                :key="s.stage"
                class="flex items-center gap-2 text-xs"
              >
                <span class="w-16 shrink-0 text-gray-500">{{ stageLabels[s.stage] ?? s.stage }}</span>
                <div class="flex-1 h-1 overflow-hidden rounded-full bg-gray-200">
                  <div
                    class="h-full bg-primary-500 transition-all duration-200"
                    :style="{ width: s.progress + '%' }"
                  />
                </div>
                <span
                  class="w-16 text-right shrink-0"
                  :class="s.progress >= 100 ? 'text-green-600' : s.progress > 0 ? 'text-gray-700' : 'text-gray-400'"
                >
                  {{ s.progress >= 100 ? '完成' : s.progress > 0 ? Math.round(s.progress) + '%' : '暂未开始' }}
                </span>
              </div>
            </div>
          </Collapse>
          <div v-if="props.completed && props.snapshot.outputPath" class="mt-3 flex items-center gap-3">
            <Tooltip :text="props.snapshot.outputPath" overflow-only class="flex-1 min-w-0">
              <span class="block truncate text-xs text-gray-500">{{ props.snapshot.outputPath }}</span>
            </Tooltip>
            <Button type="ghost" size="small" @click="handleOpenDir">打开所在目录</Button>
          </div>
          <div v-if="props.completed && props.snapshot.modName" class="mt-2 flex items-center gap-2 text-xs">
            <span class="text-gray-500">模组中文名</span>
            <span class="text-gray-800">{{ props.snapshot.modName.name }}</span>
            <Tag size="small" color="primary">{{ modNameSourceLabels[props.snapshot.modName.source] ?? props.snapshot.modName.source }}</Tag>
          </div>
          <div v-if="props.completed && props.snapshot.report" class="mt-1 text-xs text-gray-500">
            语言条目 {{ props.snapshot.report.languageAccepted }}/{{ props.snapshot.report.languageAttempted }}；class 文本 {{ props.snapshot.report.classResolved }}/{{ props.snapshot.report.classTotal }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
