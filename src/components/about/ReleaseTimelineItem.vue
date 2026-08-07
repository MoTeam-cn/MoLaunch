<script setup lang="ts">
import { ChevronDownIcon } from '@heroicons/vue/24/outline'
import Collapse from '@/components/common/Collapse.vue'
import Tag from '@/components/common/Tag.vue'
import { handleMarkdownLinkClick } from '@/utils/markdown'
import {
  CHANNEL_LABELS,
  prefixStyle,
  type ReleaseSegment,
} from '@/components/about/releaseTimeline'

defineProps<{
  segment: ReleaseSegment
  index: number
  collapsed: boolean
}>()

const emit = defineEmits<{
  toggle: []
}>()
</script>

<template>
  <li class="timeline-item">
    <span class="timeline-dot" :class="{ 'is-latest': index === 0 }" />
    <div
      class="timeline-head"
      role="button"
      tabindex="0"
      @click="emit('toggle')"
      @keydown.enter.prevent="emit('toggle')"
      @keydown.space.prevent="emit('toggle')"
    >
      <span class="timeline-chevron">
        <ChevronDownIcon
          class="h-3.5 w-3.5 transition-transform duration-300 ease-in-out"
          :class="collapsed ? '-rotate-90' : ''"
        />
      </span>
      <Tag size="small" :color="index === 0 ? 'primary' : 'gray'" class="timeline-version">
        v{{ segment.version }}
      </Tag>
      <Tag v-if="segment.channel && segment.channel !== 'stable'" size="small" color="gold">
        {{ CHANNEL_LABELS[segment.channel] }}
      </Tag>
      <Tag v-if="index === 0" size="small" color="primary">最新</Tag>
    </div>
    <Collapse :open="!collapsed">
      <ul v-if="segment.hasListItems && segment.items.length" class="commit-list">
        <li v-for="(item, itemIndex) in segment.items" :key="itemIndex" class="commit-item">
          <Tag v-if="item.prefix" size="small" :color="prefixStyle(item.prefix).color">
            {{ prefixStyle(item.prefix).label }}
          </Tag>
          <!-- eslint-disable-next-line vue/no-v-html -- renderMarkdown 已用 DOMPurify 消毒；链接点击由 handleMarkdownLinkClick 走系统浏览器 -->
          <div class="markdown-body text-xs text-gray-600 leading-relaxed" @click="handleMarkdownLinkClick" v-html="item.html" />
        </li>
      </ul>
      <!-- eslint-disable-next-line vue/no-v-html -- renderMarkdown 已用 DOMPurify 消毒；链接点击由 handleMarkdownLinkClick 走系统浏览器 -->
      <div v-else-if="!segment.hasListItems" class="markdown-body text-xs text-gray-600 leading-relaxed" @click="handleMarkdownLinkClick" v-html="segment.html" />
    </Collapse>
  </li>
</template>

<style scoped>
.timeline-item { position: relative; padding-left: 1.375rem; padding-bottom: 0.875rem; }
.timeline-item::before { content: ''; position: absolute; left: 0.25rem; top: 1rem; bottom: 0; width: 1px; background-color: #e5e6eb; }
.timeline-item:last-child { padding-bottom: 1.25rem; }
.timeline-dot { position: absolute; left: 0; top: 0.3125rem; width: 0.5625rem; height: 0.5625rem; border-radius: 9999px; background-color: #ffffff; border: 2px solid #d0d5dd; box-sizing: border-box; }
.timeline-dot.is-latest { background-color: var(--color-primary-500, #4f6ef2); border-color: var(--color-primary-500, #4f6ef2); }
.timeline-head { display: flex; align-items: center; gap: 0.375rem; margin-bottom: 0.375rem; cursor: pointer; user-select: none; border-radius: 0.375rem; padding: 0.125rem 0; transition: background-color 0.15s; }
.timeline-head:hover { background-color: #f5f6f8; }
.timeline-head:focus-visible { outline: 2px solid var(--color-primary-500, #165dff); outline-offset: 1px; }
.timeline-chevron { display: inline-flex; flex: none; color: #c0c4cc; }
.timeline-version { font-weight: 600; }
.commit-list { list-style: none; margin: 0; padding: 0; }
.commit-item { display: flex; align-items: flex-start; gap: 0.375rem; margin-bottom: 0.25rem; }
.commit-item::before { content: ''; flex: none; width: 0.25rem; height: 0.25rem; margin-top: 0.4375rem; border-radius: 9999px; background-color: #c0c4cc; }
.commit-item:last-child { margin-bottom: 0; }
.commit-item .markdown-body { flex: 1; min-width: 0; }
.commit-item :deep(.tag) { flex: none; margin-top: 0.125rem; }
.markdown-body :deep(p) { margin: 0 0 0.375rem; }
.markdown-body :deep(p:last-child) { margin-bottom: 0; }
.markdown-body :deep(h1), .markdown-body :deep(h2), .markdown-body :deep(h3), .markdown-body :deep(h4) { margin: 0.5rem 0 0.25rem; font-size: 0.8125rem; font-weight: 600; color: #1d2129; }
.markdown-body :deep(ul), .markdown-body :deep(ol) { margin: 0.125rem 0 0.375rem; padding-left: 1.125rem; list-style: disc; }
.markdown-body :deep(ol) { list-style: decimal; }
.markdown-body :deep(li) { margin: 0.125rem 0; }
.markdown-body :deep(code) { padding: 0.0625rem 0.25rem; border-radius: 0.25rem; background-color: #e5e6eb; font-family: inherit; }
.markdown-body :deep(pre) { margin: 0.375rem 0; padding: 0.5rem 0.625rem; overflow-x: auto; border-radius: 0.375rem; background-color: #f2f3f5; }
.markdown-body :deep(pre code) { padding: 0; background-color: transparent; }
.markdown-body :deep(a) { color: var(--color-primary-500, #4f6ef2); text-decoration: underline; }
</style>
