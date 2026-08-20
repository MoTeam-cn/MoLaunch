<script setup lang="ts">
/**
 * 模组翻译 - 分析结果区（左右分栏）：左侧内容区 + 右侧操作区。
 * 设置项经 defineModel 双向绑定，操作经 emit 上报父组件。
 */
import { defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const ModTranslationAnalysis = defineAsyncComponent(() => import('./ModTranslationAnalysis.vue'))
const ModTranslationSettings = defineAsyncComponent(() => import('./ModTranslationSettings.vue'))
const ModTranslationProgress = defineAsyncComponent(() => import('./ModTranslationProgress.vue'))
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

/** i18n 翻译模组/资源包（MC百科） */
const MCMOD_I18N_URL = 'https://www.mcmod.cn/class/1188.html'
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
      <ModTranslationAnalysis :analyze-result="props.analyzeResult" />

      <!-- 右侧：操作区（固定宽度，超高内部滚动） -->
      <div class="w-80 shrink-0 min-h-0 flex flex-col gap-4 overflow-y-auto">
        <!-- 翻译设置 -->
        <ModTranslationSettings
          v-if="!props.running"
          v-model:model="model"
          v-model:batch-size="batchSize"
          v-model:generate-mod-name="generateModName"
          v-model:repair-enabled="repairEnabled"
          v-model:class-text-enabled="classTextEnabled"
          :model-options="props.modelOptions"
          @start="emit('start')"
        />

        <!-- 任务进度 -->
        <ModTranslationProgress
          :snapshot="props.snapshot"
          :running="props.running"
          :completed="props.completed"
          :task-fake-progress="props.taskFakeProgress"
          @cancel="emit('cancel')"
        />
      </div>
    </div>
  </div>
</template>