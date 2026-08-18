<script setup lang="ts">
/**
 * 实验性 - 模组翻译
 *
 * 流程：选择 JAR → 后端安全解包并分析语言源 → 选择模型与批次 → 启动批量翻译 →
 * 进度经事件实时刷新 → 完成后同目录输出 `<原名>-zh_cn.jar`。
 * 单任务模型：翻译进行中不允许重新分析或再次启动。
 */
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
const Collapse = defineAsyncComponent(() => import('@/components/common/Collapse.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { ArrowUpTrayIcon } from '@heroicons/vue/24/outline'
import { pickFile } from '@/utils/fileDialog'
import { safeCall } from '@/utils/async'
import { toastError, toastSuccess, toastInfo } from '@/utils/toast'
import * as tauri from '@/utils/tauri'
import { aiLoadConfig } from '@/utils/api/ai'
import { useModTranslation } from '@/composables/useModTranslation'

const { analyzing, analyzeResult, snapshot, dragging, running, completed, analyze, start, cancel, reset, initDragDrop } = useModTranslation()

const model = ref('')
const modelOptions = ref<{ label: string; value: string }[]>([])
const batchSize = ref(40)
const batchOptions = [
  { label: '20 条/批（更稳）', value: 20 },
  { label: '40 条/批（推荐）', value: 40 },
  { label: '80 条/批（更快）', value: 80 },
]
const generateModName = ref(true)
const repairEnabled = ref(true)
const classTextEnabled = ref(true)
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
  const s = snapshot.value
  if (!s) return ''
  if (s.status === 'completed') return s.message || '翻译完成'
  if (s.status === 'failed') return s.error || '任务失败'
  if (s.status === 'cancelled') return '任务已取消'
  return s.message || '翻译中...'
})

/** 分析指定 JAR 路径（点击选择与拖入共用） */
async function analyzeFile(path: string): Promise<void> {
  reset()
  const ok = await analyze(path)
  if (ok) toastSuccess(`分析完成，共 ${analyzeResult.value?.totalEntries ?? 0} 个待翻译条目`)
}

async function handlePick() {
  const file = await pickFile({
    title: '选择要翻译的模组 JAR',
    filters: [{ name: 'JAR 文件', extensions: ['jar'] }],
  })
  if (!file) return
  await analyzeFile(file)
}

async function handleStart() {
  if (!model.value) {
    toastInfo('请先选择翻译模型')
    return
  }
  if (await start(model.value, batchSize.value, { generateModName: generateModName.value, repairEnabled: repairEnabled.value, classTextEnabled: classTextEnabled.value })) {
    toastSuccess('翻译任务已启动')
  }
}

async function handleCancel() {
  await cancel()
}

async function handleOpenDir() {
  if (!snapshot.value?.outputPath) return
  try {
    await tauri.openPath(snapshot.value.outputPath.replace(/\\[^\\/]+$/, ''))
  } catch (e) {
    toastError('打开目录失败：' + e)
  }
}

onMounted(async () => {
  const config = await safeCall(() => aiLoadConfig(), 'load ai config')
  if (config) {
    modelOptions.value = (config.models ?? []).map((m) => ({ label: m, value: m }))
    model.value = config.defaultModel ?? ''
  }
  initDragDrop(analyzeFile)
})
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="选择模组 JAR 后，后端会安全解包并识别其中的英文语言文件；翻译使用实验性 AI 服务（需先在「AI 设置」配置），完成后在同目录输出「原名-zh_cn.jar」。" />

    <!-- 选择 JAR -->
    <div class="bg-white rounded-lg border border-gray-300 p-5">
      <h3 class="text-sm font-semibold text-gray-900 mb-3">1. 选择模组 JAR</h3>
      <div
        :class="[dragging ? 'border-primary-500 bg-primary-50' : 'border-gray-300', 'flex flex-col items-center justify-center gap-2 rounded-lg border-2 border-dashed px-5 py-8 transition-colors']"
      >
        <ArrowUpTrayIcon class="h-6 w-6 text-gray-400" aria-hidden="true" />
        <p class="text-sm text-gray-600">将 JAR 文件拖入此处，或点击下方按钮选择</p>
        <Button type="outline" size="default" :disabled="running" :loading="analyzing" @click="handlePick">
          {{ analyzeResult ? '重新选择 JAR' : '选择 JAR 文件' }}
        </Button>
        <span v-if="analyzeResult" class="text-sm text-gray-700 truncate">{{ analyzeResult.filename }}</span>
      </div>
    </div>

    <!-- 分析结果 -->
    <div v-if="analyzeResult" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <div class="px-5 pt-4 pb-3 border-b border-gray-100">
        <h3 class="text-sm font-semibold text-gray-900">2. 分析结果</h3>
      </div>
      <div class="px-5 py-3 space-y-2 text-sm">
        <div class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">加载器</span>
          <span class="text-gray-800">{{ loaderLabels[analyzeResult.loader] ?? analyzeResult.loader }}</span>
        </div>
        <div v-if="analyzeResult.modIds.length" class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">Mod ID</span>
          <span class="text-gray-800">{{ analyzeResult.modIds.join(', ') }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">条目数</span>
          <span class="text-gray-800">{{ analyzeResult.totalEntries }}</span>
        </div>
        <div v-if="analyzeResult.version" class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">版本</span>
          <span class="text-gray-800">{{ analyzeResult.version }}</span>
        </div>
        <div v-if="analyzeResult.classCandidates.length" class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">class 文本</span>
          <span class="text-gray-800">{{ analyzeResult.classCandidates.length }} 个候选</span>
        </div>
        <div v-if="analyzeResult.signed" class="flex items-center gap-2">
          <span class="text-yellow-600">JAR 含签名文件，重打包后签名将失效</span>
        </div>
        <div v-for="warn in analyzeResult.warnings" :key="warn" class="flex items-center gap-2">
          <span class="text-yellow-600">{{ warn }}</span>
        </div>
      </div>

      <div class="px-5 pb-4">
        <div class="border border-gray-200 rounded overflow-hidden">
          <div class="bg-gray-50 px-3 py-2 text-xs text-gray-500 flex items-center gap-3 border-b border-gray-200">
            <span class="w-32 shrink-0">类型</span>
            <span class="flex-1 truncate">文件</span>
            <span class="w-16 text-right shrink-0">待译条目</span>
          </div>
          <div class="max-h-48 overflow-y-auto divide-y divide-gray-100">
            <div
              v-for="source in analyzeResult.sources"
              :key="source.sourcePath"
              class="px-3 py-2 text-xs flex items-center gap-3"
            >
              <span class="w-32 shrink-0 text-gray-500">{{ kindLabels[source.kind] ?? source.kind }}</span>
              <span class="flex-1 truncate text-gray-700">{{ source.targetPath }}</span>
              <span class="w-16 text-right shrink-0 text-gray-700">{{ source.entries }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="px-5 pb-4">
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
          <div class="mt-2 space-y-3">
            <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-gray-600">
              <div>预估 token：{{ analyzeResult.quote.estimatedTokens }}</div>
              <div>调用次数：{{ analyzeResult.quote.estimatedCalls }}</div>
              <div>语言批次：{{ analyzeResult.quote.languageBatches }}</div>
              <div>class 批次：{{ analyzeResult.quote.classBatches }}</div>
              <div>预估点数：{{ analyzeResult.quote.points }}</div>
            </div>
            <div class="border border-gray-200 rounded overflow-hidden">
              <div class="max-h-40 overflow-y-auto divide-y divide-gray-100">
                <div
                  v-for="item in analyzeResult.coverage"
                  :key="item.path"
                  class="px-3 py-1.5 text-xs flex items-center gap-2"
                >
                  <span class="flex-1 truncate text-gray-700">{{ item.path }}</span>
                  <Tag size="small" color="gray">{{ dispositionLabels[item.disposition] ?? item.disposition }}</Tag>
                </div>
              </div>
            </div>
          </div>
        </Collapse>
      </div>
    </div>

    <!-- 翻译设置与启动 -->
    <div v-if="analyzeResult && !running" class="bg-white rounded-lg border border-gray-300 p-5">
      <h3 class="text-sm font-semibold text-gray-900 mb-3">3. 翻译设置</h3>
      <div class="space-y-3">
        <div class="flex items-center gap-3">
          <span class="text-sm text-gray-500 w-16 shrink-0">模型</span>
          <Select v-model="model" :options="modelOptions" placeholder="选择翻译模型" />
        </div>
        <div class="flex items-center gap-3">
          <span class="text-sm text-gray-500 w-16 shrink-0">批次</span>
          <Select v-model="batchSize" :options="batchOptions" />
        </div>
        <div class="flex items-center gap-3">
          <span class="text-sm text-gray-500 w-16 shrink-0">选项</span>
          <div class="flex items-center gap-4">
            <Checkbox v-model="generateModName">生成中文名</Checkbox>
            <Checkbox v-model="repairEnabled">质量回修</Checkbox>
            <Checkbox v-model="classTextEnabled">class 文本</Checkbox>
          </div>
        </div>
        <div class="pt-2">
          <Button type="primary" @click="handleStart">开始翻译</Button>
        </div>
      </div>
    </div>

    <!-- 任务进度 -->
    <div v-if="snapshot && snapshot.status !== 'idle'" class="bg-white rounded-lg border border-gray-300 p-5">
      <div class="flex items-center justify-between gap-2 mb-2">
        <span class="text-sm font-semibold text-gray-900">{{ statusText }}</span>
        <div class="flex items-center gap-2">
          <span v-if="running" class="text-xs text-gray-500">{{ Math.round(snapshot.progress) }}%</span>
          <Button v-if="running" type="ghost" size="small" @click="handleCancel">取消</Button>
        </div>
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-gray-200">
        <div
          class="h-full bg-primary-500 transition-all duration-200"
          :style="{ width: snapshot.progress + '%' }"
        />
      </div>
      <div v-if="completed && snapshot.outputPath" class="mt-3 flex items-center gap-3">
        <span class="text-xs text-gray-500 truncate">{{ snapshot.outputPath }}</span>
        <Button type="ghost" size="small" @click="handleOpenDir">打开所在目录</Button>
      </div>
      <div v-if="completed && snapshot.modName" class="mt-2 flex items-center gap-2 text-xs">
        <span class="text-gray-500">模组中文名</span>
        <span class="text-gray-800">{{ snapshot.modName.name }}</span>
        <Tag size="small" color="primary">{{ modNameSourceLabels[snapshot.modName.source] ?? snapshot.modName.source }}</Tag>
      </div>
      <div v-if="completed && snapshot.report" class="mt-1 text-xs text-gray-500">
        语言条目 {{ snapshot.report.languageAccepted }}/{{ snapshot.report.languageAttempted }}；class 文本 {{ snapshot.report.classResolved }}/{{ snapshot.report.classTotal }}
      </div>
    </div>
  </div>
</template>
