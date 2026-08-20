<script setup lang="ts">
/**
 * 模组翻译 - 任务进度区（右侧操作区）
 */
import { computed, ref, watch, onUnmounted, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Collapse = defineAsyncComponent(() => import('@/components/common/Collapse.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import * as tauri from '@/utils/tauri'
import { toastError } from '@/utils/toast'
import type { ModTranslationTaskSnapshot } from '@/utils/api/experimental-mod-translation'

const props = defineProps<{
  snapshot: ModTranslationTaskSnapshot | null
  running: boolean
  completed: boolean
  taskFakeProgress: number
}>()

const emit = defineEmits<{ cancel: [] }>()

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
  <div v-if="snapshot && snapshot.status !== 'idle'" class="bg-white rounded-lg border border-gray-300 p-5">
    <div class="flex items-center justify-between gap-2 mb-2">
      <span class="text-sm font-semibold" :class="failed ? 'text-red-600' : 'text-gray-900'">
        {{ statusText }}<span v-if="running" class="text-primary-500">{{ '.'.repeat(dotCount) }}</span>
      </span>
      <div class="flex items-center gap-2">
        <span v-if="running" class="text-xs text-gray-500">{{ Math.round(displayProgress) }}%</span>
        <Button v-if="running" type="ghost" size="small" @click="emit('cancel')">取消</Button>
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
        <span v-if="running && snapshot.retry" class="text-yellow-600">
          第 {{ snapshot.retry.attempt }}/{{ snapshot.retry.total }} 次重试
        </span>
        <button
          v-if="snapshot.stages.length"
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
          v-for="s in snapshot.stages"
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
    <div v-if="completed && snapshot.outputPath" class="mt-3 flex items-center gap-3">
      <Tooltip :text="snapshot.outputPath" overflow-only class="flex-1 min-w-0">
        <span class="block truncate text-xs text-gray-500">{{ snapshot.outputPath }}</span>
      </Tooltip>
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
</template>