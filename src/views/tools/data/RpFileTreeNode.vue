<script setup lang="ts">
/**
 * 资源包文件树 - 递归节点
 *
 * 根节点只渲染 children（根不参与折叠），目录行点击切换展开，文件行点击选中。
 */
import { defineAsyncComponent } from 'vue'
import { formatBytes } from '@/utils/format'
import type { RpTreeNode } from '@/utils/api/tools'
import {
  ChevronRightIcon,
  CodeBracketIcon,
  DocumentIcon,
  DocumentTextIcon,
  FolderIcon,
  FolderOpenIcon,
  LanguageIcon,
  PhotoIcon,
  SpeakerWaveIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  node: RpTreeNode
  selectedPath: string
  expandedSet: Set<string>
}>()
const emit = defineEmits<{
  (e: 'select', node: RpTreeNode): void
  (e: 'toggle', relPath: string): void
}>()

const RpFileTreeNode = defineAsyncComponent(() => import('./RpFileTreeNode.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

function isExpanded(relPath: string) {
  return props.expandedSet.has(relPath)
}

function fileIcon(fileType: string) {
  switch (fileType) {
    case 'png':
      return PhotoIcon
    case 'json':
    case 'model':
      return CodeBracketIcon
    case 'lang':
      return LanguageIcon
    case 'ogg':
      return SpeakerWaveIcon
    case 'mcmeta':
    case 'text':
      return DocumentTextIcon
    default:
      return DocumentIcon
  }
}

function handleClick(child: RpTreeNode) {
  if (child.kind === 'dir') emit('toggle', child.rel_path)
  else emit('select', child)
}
</script>

<template>
  <div>
    <div v-for="child in node.children" :key="child.rel_path">
      <Tooltip :text="child.rel_path" block>
        <div
          class="flex cursor-pointer select-none items-center gap-1 rounded px-1.5 py-0.5 text-sm"
          :class="
            child.kind === 'file' && child.rel_path === selectedPath
              ? 'bg-blue-100 text-blue-700'
              : 'text-gray-700 hover:bg-gray-100'
          "
          @click="handleClick(child)"
        >
          <ChevronRightIcon
            v-if="child.kind === 'dir'"
            class="h-3.5 w-3.5 shrink-0 text-gray-400 transition-transform"
            :class="isExpanded(child.rel_path) ? 'rotate-90' : ''"
          />
          <span v-else class="w-3.5 shrink-0" />
          <component
            :is="child.kind === 'dir' ? (isExpanded(child.rel_path) ? FolderOpenIcon : FolderIcon) : fileIcon(child.file_type)"
            class="h-4 w-4 shrink-0 text-gray-400"
          />
          <span class="truncate">{{ child.name }}</span>
          <span
            v-if="child.animated"
            class="ml-1 shrink-0 rounded bg-purple-100 px-1 text-[10px] leading-4 text-purple-600"
          >动画</span>
          <span v-if="child.kind === 'file'" class="ml-auto shrink-0 pl-1 text-[10px] text-gray-400">
            {{ formatBytes(child.size) }}
          </span>
        </div>
      </Tooltip>
      <div
        v-if="child.kind === 'dir' && isExpanded(child.rel_path)"
        class="ml-3 border-l border-gray-200 pl-1.5"
      >
        <RpFileTreeNode
          :node="child"
          :selected-path="selectedPath"
          :expanded-set="expandedSet"
          @select="(n) => emit('select', n)"
          @toggle="(p) => emit('toggle', p)"
        />
      </div>
    </div>
  </div>
</template>
