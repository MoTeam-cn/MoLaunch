<script setup lang="ts">
/**
 * 资源包编辑器 - 文件树面板（搜索过滤 + 展开状态管理）
 */
import { computed, onMounted, ref, watch, defineAsyncComponent } from 'vue'
const RpFileTreeNode = defineAsyncComponent(() => import('./RpFileTreeNode.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import { collectExpandPaths, filterTreeNode, normalizeKeyword } from '@/utils/resourcepack/filterTree'
import { MagnifyingGlassIcon } from '@heroicons/vue/24/outline'
import type { RpTreeNode } from '@/utils/api/tools'

const props = defineProps<{
  tree: RpTreeNode
  selectedPath: string
}>()

const emit = defineEmits<{
  select: [node: RpTreeNode]
}>()

const searchQuery = ref('')
const expandedSet = ref<Set<string>>(new Set())

// 挂载时默认展开顶层目录（父组件以 key 区分不同包，切换包会重新挂载）
onMounted(() => {
  expandedSet.value = new Set(
    props.tree.children.filter((c) => c.kind === 'dir').map((c) => c.rel_path),
  )
})

/** 过滤后的文件树（无关键字时返回原树） */
const filteredTree = computed(() => {
  const kw = normalizeKeyword(searchQuery.value)
  return filterTreeNode(props.tree, kw)
})

watch(searchQuery, (v) => {
  const kw = normalizeKeyword(v)
  if (!kw) return
  // 搜索时自动展开所有命中路径的祖先目录
  const paths = collectExpandPaths(props.tree, kw)
  const next = new Set(expandedSet.value)
  paths.forEach((p) => next.add(p))
  expandedSet.value = next
})

function toggleNode(relPath: string) {
  const next = new Set(expandedSet.value)
  if (next.has(relPath)) next.delete(relPath)
  else next.add(relPath)
  expandedSet.value = next
}
</script>

<template>
  <div class="max-h-[400px] overflow-y-auto p-2 md:border-r md:border-gray-200">
    <div class="mb-2 px-1">
      <Input v-model="searchQuery" placeholder="搜索文件…" clearable size="small">
        <template #prefix>
          <MagnifyingGlassIcon class="h-4 w-4 text-gray-400" />
        </template>
      </Input>
    </div>
    <RpFileTreeNode
      v-if="filteredTree"
      :node="filteredTree"
      :selected-path="selectedPath"
      :expanded-set="expandedSet"
      @select="(n) => emit('select', n)"
      @toggle="toggleNode"
    />
    <p v-else class="px-1 py-4 text-center text-xs text-gray-400">未找到匹配的文件</p>
  </div>
</template>