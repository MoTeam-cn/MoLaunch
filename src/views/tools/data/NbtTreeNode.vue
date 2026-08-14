<script setup lang="ts">
/**
 * NBT 树节点（递归组件）
 *
 * 接收 NbtNode + path + expandedSet，递归渲染树形结构。
 * 点击容器节点（compound/list）触发 toggle 事件。
 */
import { computed, defineAsyncComponent } from 'vue'
import { ChevronRightIcon, ChevronDownIcon } from '@heroicons/vue/24/outline'
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import type { NbtNode } from '@/utils/api/tools'

const props = defineProps<{
  node: NbtNode
  path: string
  expandedSet: Set<string>
}>()

const emit = defineEmits<{
  toggle: [key: string]
}>()

const key = computed(() => props.path + '/' + props.node.name + ':' + props.node.tag_type)
const isExpanded = computed(() => props.expandedSet.has(key.value))
const isContainer = computed(() => props.node.tag_type === 'compound' || props.node.tag_type === 'list')

function toggle() {
  if (isContainer.value) emit('toggle', key.value)
}

function tagColor(tagType: string): string {
  switch (tagType) {
    case 'compound': return 'blue'
    case 'list': return 'purple'
    case 'string': return 'green'
    case 'int':
    case 'short':
    case 'long':
    case 'byte': return 'orange'
    case 'float':
    case 'double': return 'cyan'
    case 'byte_array':
    case 'int_array':
    case 'long_array': return 'gray'
    default: return 'gray'
  }
}

function formatValue(node: NbtNode): string {
  if (node.value === null || node.value === undefined) return ''
  if (typeof node.value === 'string') return '"' + node.value + '"'
  if (Array.isArray(node.value)) {
    if (node.value.length <= 8) return '[' + node.value.join(', ') + ']'
    return '[' + node.value.slice(0, 8).join(', ') + ', ... ] (' + node.value.length + ' items)'
  }
  return String(node.value)
}
</script>

<template>
  <div>
    <!-- 当前节点行 -->
    <div
      class="flex items-center gap-1.5 py-1 cursor-pointer hover:bg-gray-50 rounded px-1"
      @click="toggle"
    >
      <ChevronDownIcon v-if="isContainer && isExpanded" class="h-3.5 w-3.5 flex-none text-gray-400" />
      <ChevronRightIcon v-else-if="isContainer" class="h-3.5 w-3.5 flex-none text-gray-400" />
      <span v-else class="inline-block w-3.5 flex-none" />
      <Tag
        size="small"
        class="flex-none"
        :color="tagColor(node.tag_type)"
      >{{ node.tag_type }}</Tag>
      <span v-if="node.name" class="text-sm text-gray-800 font-medium">{{ node.name }}</span>
      <span v-else class="text-sm text-gray-400 italic">(unnamed)</span>
      <span
        v-if="!isContainer && node.value !== null && node.value !== undefined"
        class="text-sm text-gray-500 truncate"
      >{{ formatValue(node) }}</span>
      <span
        v-if="isContainer && node.children.length > 0"
        class="text-xs text-gray-400"
      >({{ node.children.length }})</span>
    </div>

    <!-- 子节点（展开时） -->
    <div v-if="isContainer && isExpanded" class="ml-4 border-l border-gray-100 pl-2">
      <NbtTreeNode
        v-for="(child, idx) in node.children"
        :key="idx"
        :node="child"
        :path="key"
        :expanded-set="expandedSet"
        @toggle="emit('toggle', $event)"
      />
    </div>
  </div>
</template>
