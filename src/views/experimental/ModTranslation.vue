<script setup lang="ts">
/**
 * 实验性 - 模组翻译：视图壳。
 * 三态流程：upload（铺满容器上传）→ analyzing（假进度条）→ result（左右分栏，可返回上传）。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const ModTranslationResult = defineAsyncComponent(() => import('./ModTranslationResult.vue'))
import { ArrowUpTrayIcon } from '@heroicons/vue/24/outline'
import { pickFile } from '@/utils/fileDialog'
import { safeCall } from '@/utils/async'
import { toastSuccess, toastInfo } from '@/utils/toast'
import { showConfirmAsync } from '@/utils/modal'
import { aiLoadConfig } from '@/utils/api/ai'
import { useModTranslation } from '@/composables/useModTranslation'

const {
  view,
  fakeProgress,
  taskFakeProgress,
  analyzing,
  analyzeResult,
  snapshot,
  dragging,
  running,
  completed,
  analyze,
  start,
  cancel,
  reset,
  backToUpload,
  initDragDrop,
} = useModTranslation()

const model = ref('')
const modelOptions = ref<{ label: string; value: string }[]>([])
const batchSize = ref(40)
const generateModName = ref(true)
const repairEnabled = ref(true)
const classTextEnabled = ref(true)
/** 分析中展示的文件名 */
const analyzingName = ref('')

/** 分析指定 JAR 路径（点击选择与拖入共用） */
async function analyzeFile(path: string): Promise<void> {
  analyzingName.value = path.split(/[\\/]/).pop() ?? path
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
  const existing = analyzeResult.value?.existingChinese ?? []
  if (existing.length > 0) {
    const ok = await showConfirmAsync(
      '覆盖中文语言文件',
      `该模组已包含 ${existing.length} 个中文语言文件（如 ${existing[0].path}），翻译将覆盖这些文件，是否继续？`,
      {
        messageHtml: `该模组已包含 <strong>${existing.length}</strong> 个中文语言文件（如 <strong class="font-semibold break-all">${existing[0].path}</strong>），翻译将覆盖这些文件，是否继续？`,
      },
    )
    if (!ok) return
  }
  if (await start(model.value, batchSize.value, { generateModName: generateModName.value, repairEnabled: repairEnabled.value, classTextEnabled: classTextEnabled.value })) {
    toastSuccess('翻译任务已启动')
  }
}

async function handleCancel() {
  await cancel()
}

function handleBack() {
  if (running.value) {
    toastInfo('翻译进行中，无法返回')
    return
  }
  backToUpload()
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
  <div class="h-full overflow-hidden">
    <Transition name="view-fade" mode="out-in">
      <!-- 上传区：铺满容器 -->
      <div v-if="view === 'upload'" key="upload" class="h-full">
        <div
          class="h-full flex flex-col items-center justify-center gap-4 rounded-xl border-2 border-dashed transition-colors"
          :class="dragging ? 'border-primary-500 bg-primary-50' : 'border-gray-300'"
        >
          <ArrowUpTrayIcon class="h-10 w-10 text-gray-400" aria-hidden="true" />
          <div class="text-center">
            <p class="text-sm text-gray-600">将模组 JAR 文件拖入此处</p>
            <p v-if="analyzeResult" class="mt-1 text-xs text-gray-400">已分析：{{ analyzeResult.filename }}</p>
          </div>
          <Button type="outline" size="default" :loading="analyzing" @click="handlePick">
            {{ analyzeResult ? '重新选择 JAR' : '选择 JAR 文件' }}
          </Button>
        </div>
      </div>

      <!-- 分析中：假进度条 -->
      <div v-else-if="view === 'analyzing'" key="analyzing" class="h-full flex flex-col items-center justify-center gap-4">
        <div class="h-14 w-14 rounded-full border-4 border-primary-200 border-t-primary-500 animate-spin" />
        <div class="text-center">
          <p class="text-sm text-gray-700">{{ analyzingName }}</p>
          <p class="mt-1 text-xs text-gray-400">正在分析语言文件…</p>
        </div>
        <div class="w-64">
          <div class="h-1.5 w-full overflow-hidden rounded-full bg-gray-200">
            <div class="h-full bg-primary-500 transition-all duration-300" :style="{ width: fakeProgress + '%' }" />
          </div>
          <p class="mt-1 text-right text-xs text-gray-400">{{ Math.round(fakeProgress) }}%</p>
        </div>
      </div>

      <!-- 结果区：左右分栏 -->
      <ModTranslationResult
        v-else
        key="result"
        v-model:model="model"
        v-model:batch-size="batchSize"
        v-model:generate-mod-name="generateModName"
        v-model:repair-enabled="repairEnabled"
        v-model:class-text-enabled="classTextEnabled"
        class="h-full"
        :analyze-result="analyzeResult!"
        :snapshot="snapshot"
        :running="running"
        :completed="completed"
        :task-fake-progress="taskFakeProgress"
        :model-options="modelOptions"
        @start="handleStart"
        @cancel="handleCancel"
        @back="handleBack"
      />
    </Transition>
  </div>
</template>

<style scoped>
.view-fade-enter-active,
.view-fade-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}
.view-fade-enter-from {
  opacity: 0;
  transform: translateY(8px);
}
.view-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
