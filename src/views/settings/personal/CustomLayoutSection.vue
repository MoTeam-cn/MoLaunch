<script setup lang="ts">
/**
 * 自定义布局配置区（格式 / 来源 / 内联编辑器 / URL 加载 / 示例导出）
 *
 * 由 HomePanelModeSection 在 panelMode === 'custom' 时渲染。
 * 逻辑集中在 useCustomLayout composable，此处仅保留模板 + 组装。
 */
import Select from '@/components/common/Select.vue'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import {
  ArrowPathIcon,
  ArrowDownTrayIcon,
  DocumentArrowDownIcon,
} from '@heroicons/vue/24/outline'
import { useCustomLayout } from '@/composables/useCustomLayout'

const {
  customConfig,
  inlinePlaceholder,
  formatOptions,
  sourceOptions,
  inlineContentDraft,
  onInlineContentChange,
  urlRefreshing,
  onRefreshUrl,
  onFormatChange,
  onSourceChange,
  onUrlInput,
  cachedTimeText,
  onExportSampleLayout,
  fillingTemplate,
  onFillTemplate,
} = useCustomLayout()
</script>

<template>
  <div class="space-y-4">
    <!-- 格式 + 来源（上下堆叠，避免并列时文字被截断） -->
    <div class="space-y-3">
      <div class="flex items-center justify-between gap-4">
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900">布局格式</p>
          <p class="text-xs text-gray-500 mt-0.5">JSON/XML 结构化，HTML 直接渲染</p>
        </div>
        <div class="flex-none w-40">
          <Select
            :model-value="customConfig.format"
            :options="formatOptions"
            @update:model-value="onFormatChange"
          />
        </div>
      </div>
      <div class="flex items-center justify-between gap-4">
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900">内容来源</p>
          <p class="text-xs text-gray-500 mt-0.5">内联直接编辑，URL 远程加载并缓存</p>
        </div>
        <div class="flex-none w-40">
          <Select
            :model-value="customConfig.source"
            :options="sourceOptions"
            @update:model-value="onSourceChange"
          />
        </div>
      </div>
    </div>

    <!-- 示例模板操作（填入内联编辑器 / 导出文件） -->
    <div class="flex items-center justify-between rounded border border-dashed border-gray-300 bg-white/50 px-3 py-2">
      <div class="min-w-0">
        <p class="text-xs font-medium text-gray-700">示例模板（{{ customConfig.format.toUpperCase() }}）</p>
        <p class="mt-0.5 text-[11px] text-gray-400">
          填入到内联编辑器快速开始，或导出为文件供外部编辑
        </p>
      </div>
      <div class="flex flex-none gap-2">
        <Button
          v-if="customConfig.source === 'inline'"
          type="outline"
          size="small"
          :disabled="fillingTemplate"
          @click="onFillTemplate"
        >
          <DocumentArrowDownIcon class="mr-1 h-3.5 w-3.5" />
          填入模板
        </Button>
        <Button type="outline" size="small" @click="onExportSampleLayout">
          <ArrowDownTrayIcon class="mr-1 h-3.5 w-3.5" />
          导出文件
        </Button>
      </div>
    </div>

    <!-- 内联编辑器 -->
    <div v-if="customConfig.source === 'inline'">
      <div class="mb-2 flex items-center justify-between">
        <p class="text-sm font-medium text-gray-900">
          {{ customConfig.format === 'html' ? 'HTML' : customConfig.format === 'xml' ? 'XML' : 'JSON' }} 内容
        </p>
        <span class="text-[11px] text-gray-400">编辑后自动保存（防抖 500ms）</span>
      </div>
      <Input
        v-model="inlineContentDraft"
        textarea
        :rows="16"
        resize="vertical"
        :placeholder="inlinePlaceholder"
        class="custom-layout-editor"
        @input="onInlineContentChange"
      />
    </div>

    <!-- URL 加载 -->
    <div v-else class="space-y-3">
      <div>
        <div class="mb-2 flex items-center justify-between">
          <p class="text-sm font-medium text-gray-900">布局 URL</p>
          <span class="text-[11px] text-gray-400">缓存时间：{{ cachedTimeText }}</span>
        </div>
        <div class="flex gap-2">
          <input
            :value="customConfig.url"
            type="text"
            placeholder="https://example.com/layout.json"
            class="flex-1 rounded border border-gray-300 bg-white px-3 py-1.5 text-xs text-gray-900 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            @input="onUrlInput"
          />
          <Button
            type="outline"
            size="small"
            :disabled="urlRefreshing"
            @click="onRefreshUrl"
          >
            <template #icon><ArrowPathIcon class="w-3.5 h-3.5" :class="{ 'animate-spin': urlRefreshing }" /></template>
            刷新缓存
          </Button>
        </div>
      </div>
      <p class="text-[11px] text-gray-400">
        URL 内容会下载并缓存到本地，启动器重启后自动加载缓存；点击「刷新缓存」可强制更新
      </p>
    </div>
  </div>
</template>

<style scoped>
/* 代码编辑器：等宽字体 + 小字号，对齐原原生 textarea 的代码输入体验 */
.custom-layout-editor :deep(.textarea-inner) {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12px;
}
</style>
