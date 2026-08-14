<script setup lang="ts">
/**
 * 更新日志时间线组件：将合并的多版本 Markdown 按版本拆分为时间线节点。
 * 版本与条目解析见 releaseTimeline.ts，单个节点渲染见 ReleaseTimelineItem.vue。
 */
import { reactive, computed, defineAsyncComponent } from 'vue'
const ReleaseTimelineItem = defineAsyncComponent(() => import('@/components/about/ReleaseTimelineItem.vue'))
import { handleMarkdownLinkClick } from '@/utils/markdown'
import { parseReleaseNotes } from '@/components/about/releaseTimeline'

const props = defineProps<{ notes: string }>()
const segments = computed(() => parseReleaseNotes(props.notes))
const hasVersioned = computed(() => segments.value.some((segment) => segment.version !== null))
const collapsed = reactive<Record<number, boolean>>({})

function isCollapsed(index: number): boolean {
  return collapsed[index] ?? false
}

function toggleCollapsed(index: number): void {
  collapsed[index] = !isCollapsed(index)
}
</script>

<template>
  <div v-if="segments.length">
    <!-- 无版本标题：退化为整段 Markdown 渲染（历史单段数据） -->
    <!-- eslint-disable vue/no-v-html -- renderMarkdown 已用 DOMPurify 消毒；链接点击由 handleMarkdownLinkClick 走系统浏览器 -->
    <div
      v-if="!hasVersioned"
      class="markdown-body text-xs text-gray-600 leading-relaxed"
      @click="handleMarkdownLinkClick"
      v-html="segments[0].html"
    />
    <!-- eslint-enable vue/no-v-html -->

    <!-- 时间线：左侧竖线串起各版本节点 -->
    <ol v-else class="release-timeline">
      <ReleaseTimelineItem
        v-for="(segment, index) in segments"
        :key="`${segment.version ?? 'raw'}-${index}`"
        :segment="segment"
        :index="index"
        :collapsed="isCollapsed(index)"
        @toggle="toggleCollapsed(index)"
      />
    </ol>
  </div>
</template>

<style scoped>
.release-timeline {
  list-style: none;
  margin: 0;
  padding: 0;
}

.markdown-body :deep(p) {
  margin: 0 0 0.375rem;
}

.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4) {
  margin: 0.5rem 0 0.25rem;
  font-size: 0.8125rem;
  font-weight: 600;
  color: #1d2129;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 0.125rem 0 0.375rem;
  padding-left: 1.125rem;
  list-style: disc;
}

.markdown-body :deep(ol) {
  list-style: decimal;
}

.markdown-body :deep(li) {
  margin: 0.125rem 0;
}

.markdown-body :deep(code) {
  padding: 0.0625rem 0.25rem;
  border-radius: 0.25rem;
  background-color: #e5e6eb;
  font-family: inherit;
}

.markdown-body :deep(pre) {
  margin: 0.375rem 0;
  padding: 0.5rem 0.625rem;
  overflow-x: auto;
  border-radius: 0.375rem;
  background-color: #f2f3f5;
}

.markdown-body :deep(pre code) {
  padding: 0;
  background-color: transparent;
}

.markdown-body :deep(a) {
  color: var(--color-primary-500, #4f6ef2);
  text-decoration: underline;
}
</style>
