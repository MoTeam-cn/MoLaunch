<script setup lang="ts">
/**
 * 自定义布局渲染面板
 *
 * 接收 JSON/XML 格式的布局内容，解析后渲染为启动器风格的组件。
 *
 * 工作流程：
 * 1. 解析布局内容（JSON / XML）→ LayoutSchema
 * 2. 加载数据源（cache / system / versions / history）→ DataContext
 * 3. 根据 schema 逐个渲染 section，使用 context 解析插值
 *
 * 数据刷新策略：进入页面加载一次，之后每 3 秒轮询数据源（不重新解析布局）。
 *
 * 渲染委托给 LayoutSectionRenderer 子组件，HTML shadow DOM 渲染与渲染辅助函数
 * 分别提取到 htmlShadowRenderer.ts / renderHelpers.ts。
 */
import { ref, computed, onMounted, watch } from 'vue'
import { usePolling } from '@/composables/usePolling'
import { parseJsonLayout, parseXmlLayout } from './parser'
import { loadDataContext, type DataContext } from './datasource'
import type { LayoutSchema } from './types'
import type { LayoutFormat } from '@/types/plugin'
import { safeCall } from '@/utils/async'
import {
  ChartBarIcon,
  CircleStackIcon,
  CpuChipIcon,
  ClockIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'
import LayoutSectionRenderer from './LayoutSectionRenderer.vue'
import Button from '@/components/common/Button.vue'

const props = defineProps<{
  /** 布局格式 */
  format: LayoutFormat
  /** 布局内容（JSON/XML 字符串） */
  content: string
}>()

/** 解析后的 schema */
const schema = ref<LayoutSchema | null>(null)
/** 解析错误 */
const parseError = ref<string | null>(null)
/** 数据上下文 */
const dataCtx = ref<DataContext>({})
/** 是否正在加载数据 */
const loading = ref(true)
/** 是否正在刷新 */
const refreshing = ref(false)

/** 解析布局内容 */
function parseLayout() {
  parseError.value = null
  schema.value = null

  const result = props.format === 'xml'
    ? parseXmlLayout(props.content)
    : parseJsonLayout(props.content)

  if (result.error) {
    parseError.value = result.error
  } else if (result.schema) {
    schema.value = result.schema
  }
}

/** 加载数据源 */
async function loadData() {
  const ctx = await safeCall(() => loadDataContext(), '[CustomLayout] load data context')
  if (ctx) dataCtx.value = ctx
  loading.value = false
  refreshing.value = false
}

/** 手动刷新 */
async function refresh() {
  refreshing.value = true
  await loadData()
}

/** 图标组件映射 */
const iconMap: Record<string, typeof ChartBarIcon> = {
  'chart-bar': ChartBarIcon,
  'circle-stack': CircleStackIcon,
  'cpu-chip': CpuChipIcon,
  'clock': ClockIcon,
}

/** 标题图标 */
const titleIcon = computed(() => {
  if (!schema.value?.icon) return null
  return iconMap[schema.value.icon] ?? null
})

// ==================== 生命周期 ====================

watch(() => [props.format, props.content], () => {
  parseLayout()
  loading.value = true
  loadData()
}, { immediate: false })

// 每 3 秒轮询数据源（不重新解析布局），onUnmounted 自动清理
const { start: startPolling } = usePolling(loadData, 3000)

onMounted(() => {
  parseLayout()
  loadData()
  startPolling()
})
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <!-- 解析错误 -->
    <div v-if="parseError" class="flex flex-1 flex-col items-center justify-center">
      <div class="mb-3 rounded-full bg-red-50 p-3">
        <svg class="h-8 w-8 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
        </svg>
      </div>
      <p class="text-sm font-medium text-gray-900">布局解析失败</p>
      <p class="mt-1 max-w-md text-center text-xs text-gray-500">{{ parseError }}</p>
    </div>

    <!-- 加载中 -->
    <div v-else-if="loading" class="flex flex-1 items-center justify-center text-sm text-gray-500">
      加载中...
    </div>

    <!-- 正常渲染 -->
    <template v-else-if="schema">
      <!-- 标题栏（固定） -->
      <div v-if="schema.title" class="flex flex-none items-center justify-between mb-4">
        <div class="flex items-center gap-2">
          <component
            :is="titleIcon"
            v-if="titleIcon"
            class="h-5 w-5 text-primary-500"
          />
          <h3 class="text-base font-semibold text-gray-900">{{ schema.title }}</h3>
        </div>
        <Button
          type="ghost"
          size="mini"
          :disabled="refreshing"
          @click="refresh"
        >
          <template #icon>
            <ArrowPathIcon class="h-3.5 w-3.5" :class="{ 'animate-spin': refreshing }" />
          </template>
          刷新
        </Button>
      </div>

      <!-- sections（可滚动） -->
      <div class="flex-1 space-y-4 overflow-y-auto pr-1">
        <LayoutSectionRenderer
          v-for="(section, idx) in schema.sections"
          :key="idx"
          :section="section"
          :data-ctx="dataCtx"
        />
      </div>
    </template>
  </div>
</template>
