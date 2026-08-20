<script setup lang="ts">
/**
 * 自定义布局入口
 *
 * 根据布局格式选择对应的渲染器：
 * - json / xml：使用 CustomLayoutPanel（结构化布局，启动器提供组件库）
 * - html：使用 HtmlLayoutPanel（直接渲染 HTML，iframe sandbox 隔离）
 *
 * 供 Home.vue 通过 homePanelComponent 渲染。
 */
import { computed, defineAsyncComponent } from 'vue'
const CustomLayoutPanel = defineAsyncComponent(() => import('./CustomLayoutPanel.vue'))
const HtmlLayoutPanel = defineAsyncComponent(() => import('./HtmlLayoutPanel.vue'))
import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
import { isCustomLayoutUrlAllowed } from '@/stores/plugins/plugins-layout'
import type { CustomLayoutConfig } from '@/types/plugin'

const props = defineProps<{
  /** 自定义布局配置 */
  config: CustomLayoutConfig
}>()

/** 是否远程 URL 来源 */
const isRemote = computed(() => props.config.source === 'url')
/** URL 是否不在白名单内（阻止渲染） */
const urlBlocked = computed(() => isRemote.value && !isCustomLayoutUrlAllowed(props.config.url))

/** 获取布局内容（内联或 URL 缓存；URL 不在白名单时返回空） */
const content = computed(() => {
  if (props.config.source === 'url') {
    return urlBlocked.value ? '' : (props.config.cachedContent || '')
  }
  return props.config.inlineContent
})

/** 是否使用 HTML 渲染器 */
const isHtml = computed(() => props.config.format === 'html')
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <!-- URL 不在白名单：阻止渲染 -->
    <div
      v-if="urlBlocked"
      class="flex flex-1 flex-col items-center justify-center p-6 text-center"
    >
      <ExclamationTriangleIcon class="mb-3 h-10 w-10 text-red-500" />
      <p class="text-sm font-medium text-gray-900">远程布局已阻止</p>
      <p class="mt-1 max-w-md break-all text-xs text-gray-500">
        URL 不在允许的域名白名单内（仅支持 https）：{{ config.url }}
      </p>
    </div>
    <template v-else>
      <!-- 远程布局风险警示 -->
      <div
        v-if="isRemote"
        class="flex flex-none items-center gap-1.5 border-b border-yellow-200 bg-yellow-50 px-3 py-1.5 text-xs text-yellow-700"
      >
        <ExclamationTriangleIcon class="h-3.5 w-3.5 flex-none" />
        <span class="truncate">远程布局：{{ config.url }}，请确认来源可信</span>
      </div>
      <CustomLayoutPanel
        v-if="!isHtml"
        class="min-h-0 flex-1"
        :format="config.format"
        :content="content"
      />
      <HtmlLayoutPanel
        v-else
        class="min-h-0 flex-1"
        :content="content"
      />
    </template>
  </div>
</template>
