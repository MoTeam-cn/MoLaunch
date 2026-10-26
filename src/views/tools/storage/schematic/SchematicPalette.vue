<script setup lang="ts">
/**
 * 方块调色板选择器
 *
 * 支持搜索过滤；选中后向父级回传调色板索引。
 */

import { computed, ref } from 'vue'
import { blockColor } from '@/utils/schematic/decode'

const props = defineProps<{
  palette: readonly string[]
  modelValue: number
  /** 每项数量（用于排序展示，可空） */
  counts?: Record<string, number>
}>()

const emit = defineEmits<{ 'update:modelValue': [value: number] }>()

const search = ref('')

const items = computed(() => {
  const keyword = search.value.trim().toLowerCase()
  return props.palette
    .map((id, index) => ({
      id,
      index,
      color: blockColor(id),
      count: props.counts?.[id] ?? 0,
    }))
    .filter((item) => (keyword ? item.id.toLowerCase().includes(keyword) : true))
    .sort((a, b) => b.count - a.count || a.id.localeCompare(b.id))
    .slice(0, 200)
})
</script>

<template>
  <div class="flex flex-col gap-2">
    <input
      v-model="search"
      type="text"
      placeholder="搜索方块 id"
      class="h-8 rounded border border-gray-300 px-2 text-xs focus:border-brand-2 focus:outline-none"
    />
    <div class="max-h-56 overflow-y-auto rounded border border-gray-200">
      <button
        v-for="item in items"
        :key="item.index"
        type="button"
        class="flex w-full items-center gap-2 border-b border-gray-100 px-2 py-1 text-left text-xs last:border-b-0"
        :class="modelValue === item.index ? 'bg-brand-2/10 text-brand-2' : 'hover:bg-gray-50'"
        @click="emit('update:modelValue', item.index)"
      >
        <span class="h-3 w-3 shrink-0 rounded-sm border border-gray-300" :style="{ backgroundColor: item.color }" />
        <span class="flex-1 truncate">{{ item.id.replace('minecraft:', '') }}</span>
        <span v-if="item.count" class="text-gray-400">{{ item.count.toLocaleString() }}</span>
      </button>
    </div>
  </div>
</template>
