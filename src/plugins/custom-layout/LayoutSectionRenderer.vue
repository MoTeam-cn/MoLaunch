<script setup lang="ts">
/**
 * 单个布局 section 渲染器
 *
 * 从 CustomLayoutPanel.vue 提取，负责根据 section.type 渲染对应 UI：
 * - stat-grid：统计网格
 * - list：数据列表
 * - progress：进度条
 * - text：文本块
 * - divider：分割线
 * - html：shadow DOM 自定义 HTML
 *
 * 渲染计算逻辑复用 renderHelpers.ts 纯函数，HTML 渲染复用 htmlShadowRenderer.ts。
 */
import { formatValue, type DataContext } from './datasource'
import type { LayoutSection } from './types'
import {
  colorClassMap,
  progressBarColorMap,
  textVariantMap,
  resolveStatValue,
  resolveProgressValue,
  resolveProgressMax,
  progressPercent,
  getListEntries,
  formatFieldValue,
} from './renderHelpers'
import { renderHtmlShadow } from './htmlShadowRenderer'

const props = defineProps<{
  /** 当前 section 配置 */
  section: LayoutSection
  /** 数据上下文（用于解析插值） */
  dataCtx: DataContext
}>()
</script>

<template>
  <!-- 统计网格 -->
  <div
    v-if="props.section.type === 'stat-grid'"
    class="grid gap-3"
    :style="{ gridTemplateColumns: `repeat(${props.section.columns || 3}, minmax(0, 1fr))` }"
  >
    <div
      v-for="(item, itemIdx) in props.section.items"
      :key="itemIdx"
      class="rounded-md border border-gray-200 p-3"
    >
      <p class="text-[11px] text-gray-500">{{ item.label }}</p>
      <p
        class="mt-1 text-lg font-semibold"
        :class="item.color ? colorClassMap[item.color] : 'text-gray-900'"
      >
        {{ resolveStatValue(item, props.dataCtx) }}
      </p>
    </div>
  </div>

  <!-- 数据列表 -->
  <div
    v-else-if="props.section.type === 'list'"
    class="rounded-md border border-gray-200 p-4"
  >
    <div v-if="props.section.title" class="mb-3 flex items-center justify-between">
      <span class="text-sm font-medium text-gray-900">{{ props.section.title }}</span>
      <span class="text-xs text-gray-500">{{ getListEntries(props.section, props.dataCtx).length }} 条</span>
    </div>
    <div class="space-y-2">
      <div
        v-for="(entry, entryIdx) in getListEntries(props.section, props.dataCtx)"
        :key="entryIdx"
        class="flex items-center justify-between rounded bg-gray-50 px-3 py-2"
      >
        <div class="min-w-0 flex-1">
          <template v-for="(field, fieldIdx) in props.section.fields" :key="fieldIdx">
            <span
              v-if="fieldIdx > 0"
              class="mx-1.5 text-gray-300"
            >·</span>
            <span v-if="field.label" class="text-[10px] text-gray-400">{{ field.label }}: </span>
            <span class="text-xs text-gray-900">{{ formatFieldValue(entry, field) }}</span>
          </template>
        </div>
      </div>
      <p
        v-if="getListEntries(props.section, props.dataCtx).length === 0"
        class="py-2 text-center text-xs text-gray-400"
      >
        暂无数据
      </p>
    </div>
  </div>

  <!-- 进度条 -->
  <div
    v-else-if="props.section.type === 'progress'"
    class="rounded-md border border-gray-200 p-4"
  >
    <div v-if="props.section.label" class="mb-2 flex items-center justify-between">
      <span class="text-xs font-medium text-gray-900">{{ props.section.label }}</span>
      <span class="text-xs text-gray-500">
        {{ formatValue(resolveProgressValue(props.section.value, props.dataCtx), props.section.format || 'text') }}
        <span v-if="props.section.max"> / {{ formatValue(resolveProgressMax(props.section.max, props.dataCtx), props.section.format || 'text') }}</span>
      </span>
    </div>
    <div v-else class="mb-2 flex items-center justify-end">
      <span class="text-xs text-gray-500">
        {{ formatValue(resolveProgressValue(props.section.value, props.dataCtx), props.section.format || 'text') }}
        <span v-if="props.section.max"> / {{ formatValue(resolveProgressMax(props.section.max, props.dataCtx), props.section.format || 'text') }}</span>
      </span>
    </div>
    <div class="h-2 w-full overflow-hidden rounded-full bg-gray-100">
      <div
        class="h-full transition-all duration-500"
        :class="props.section.color ? progressBarColorMap[props.section.color] : 'bg-primary-500'"
        :style="{ width: `${progressPercent(props.section, props.dataCtx)}%` }"
      />
    </div>
  </div>

  <!-- 文本块 -->
  <p
    v-else-if="props.section.type === 'text'"
    class="text-xs"
    :class="props.section.variant ? textVariantMap[props.section.variant] : 'text-gray-700'"
  >
    {{ props.section.content }}
  </p>

  <!-- 分割线 -->
  <hr v-else-if="props.section.type === 'divider'" class="border-gray-200" />

  <!-- 自定义 HTML（shadow DOM 渲染，CSS 隔离 + 内联 JS/CSS 支持） -->
  <div
    v-else-if="props.section.type === 'html'"
    :ref="(el) => { if (el) renderHtmlShadow(el as HTMLElement, props.section) }"
    :style="{ height: (props.section.height || 200) + 'px' }"
    class="w-full overflow-hidden rounded-md border border-gray-200"
  />
</template>
