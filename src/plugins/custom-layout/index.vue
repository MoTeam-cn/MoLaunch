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
import type { CustomLayoutConfig } from '@/types/plugin'

const props = defineProps<{
  /** 自定义布局配置 */
  config: CustomLayoutConfig
}>()

/** 获取布局内容（内联或 URL 缓存） */
const content = computed(() => {
  if (props.config.source === 'url') {
    return props.config.cachedContent || ''
  }
  return props.config.inlineContent
})

/** 是否使用 HTML 渲染器 */
const isHtml = computed(() => props.config.format === 'html')
</script>

<template>
  <CustomLayoutPanel
    v-if="!isHtml"
    :format="config.format"
    :content="content"
  />
  <HtmlLayoutPanel
    v-else
    :content="content"
  />
</template>
