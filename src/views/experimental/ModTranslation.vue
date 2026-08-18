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
import { pickFile } from '@/utils/fileDialog'
import { safeCall } from '@/utils/async'
import { toastError, toastSuccess, toastInfo } from '@/utils/toast'
import * as tauri from '@/utils/tauri'
import { aiLoadConfig } from '@/utils/api/ai'
import { useModTranslation } from '@/composables/useModTranslation'

const { analyzing, analyzeResult, snapshot, running, completed, analyze, start, cancel, reset } =
  useModTranslation()

const model = ref('')
const modelOptions = ref<{ label: string; value: string }[]>([])
const batchSize = ref(40)
const batchOptions = [
  { label: '20 条/批（更稳）', value: 20 },
  { label: '40 条/批（推荐）', value: 40 },
  { label: '80 条/批（更快）', value: 80 },
]

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

const statusText = computed(() => {
  if (!snapshot.value) return ''
  const s = snapshot.value
  if (s.status === 'completed') return s.message || '翻译完成'
  if (s.status === 'failed') return s.error || '任务失败'
  if (s.status === 'cancelled') return '任务已取消'
  return s.message || '翻译中...'
})

async function handlePick() {
  const file = await pickFile({
    title: '选择要翻译的模组 JAR',
    filters: [{ name: 'JAR 文件', extensions: ['jar'] }],
  })
  if (!file) return
  reset()
  const ok = await analyze(file)
  if (ok) toastSuccess(`分析完成，共 ${analyzeResult.value?.totalEntries ?? 0} 个待翻译条目`)
}

async function handleStart() {
  if (!model.value) {
    toastInfo('请先选择翻译模型')
    return
  }
  if (await start(model.value, batchSize.value)) {
    toastSuccess('翻译任务已启动')
  }
}

async function handleCancel() {
  await cancel()
}

async function handleOpenDir() {
  if (!snapshot.value?.outputPath) return
  const dir = snapshot.value.outputPath.replace(/\\[^\\/]+$/, '')
  try {
    await tauri.openPath(dir)
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
})
</script>

<template>
  <div class="space-y-4">
    <AlertV2
      type="info"
      message="选择模组 JAR 后，后端会安全解包并识别其中的英文语言文件；翻译使用实验性 AI 服务（需先在「AI 设置」配置），完成后在同目录输出「原名-zh_cn.jar」。"
    />

    <!-- 选择 JAR -->
    <div class="bg-white rounded-lg border border-gray-300 p-5">
      <h3 class="text-sm font-semibold text-gray-900 mb-3">1. 选择模组 JAR</h3>
      <div class="flex items-center gap-3">
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
    </div>
  </div>
</template>
