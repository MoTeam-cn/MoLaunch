<script setup lang="ts">
/**
 * 方块调色板选择器
 *
 * 支持搜索过滤；选中后向父级回传调色板索引。
 * 条目优先显示原版方块贴图，取不到贴图时退回纯色。
 */

import { computed, ref, defineAsyncComponent } from 'vue'
import { blockBaseName, blockColor } from '@/utils/schematic/decode'
import { resolveTexture, type BlockAtlas } from '@/utils/schematic/textures'

const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))

/** 色块边长（px），与原版 16×16 贴图等比放大 */
const SWATCH_SIZE = 16

const props = defineProps<{
  palette: readonly string[]
  modelValue: number
  /** 每项数量（用于排序展示，可空） */
  counts?: Record<string, number>
  /** 方块贴图图集，为空时只显示纯色 */
  atlas?: BlockAtlas | null
}>()

const emit = defineEmits<{ 'update:modelValue': [value: number] }>()

const search = ref('')

const atlas = computed(() => props.atlas ?? null)

const items = computed(() => {
  const keyword = search.value.trim().toLowerCase()
  return props.palette
    .map((id, index) => ({
      id,
      index,
      color: blockColor(id),
      texture: resolveTexture(atlas.value, id),
      count: props.counts?.[id] ?? 0,
    }))
    .filter((item) => (keyword ? item.id.toLowerCase().includes(keyword) : true))
    .sort((a, b) => b.count - a.count || a.id.localeCompare(b.id))
    .slice(0, 200)
})

/** 有贴图时按图集坐标切图，否则退回纯色块 */
function swatchStyle(item: { color: string; texture: [number, number, number, number] | null }) {
  const sheet = atlas.value
  const rect = item.texture
  if (!sheet || !rect) return { backgroundColor: item.color }
  const scale = SWATCH_SIZE / rect[2]
  return {
    backgroundImage: `url(${sheet.url})`,
    backgroundSize: `${sheet.size[0] * scale}px ${sheet.size[1] * scale}px`,
    backgroundPosition: `-${rect[0] * scale}px -${rect[1] * scale}px`,
  }
}

/** 列表里只显示方块名，朝向等属性在完整提示中查看 */
function displayName(id: string): string {
  return blockBaseName(id).replace('minecraft:', '')
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <Input v-model="search" placeholder="搜索方块 id" clearable />
    <div class="max-h-56 overflow-y-auto rounded border border-gray-200">
      <button
        v-for="item in items"
        :key="item.index"
        type="button"
        class="flex w-full items-center gap-2 border-b border-gray-100 px-2 py-1 text-left text-xs last:border-b-0"
        :class="modelValue === item.index ? 'bg-brand-2/10 text-brand-2' : 'hover:bg-gray-50'"
        @click="emit('update:modelValue', item.index)"
      >
        <span
          class="h-4 w-4 shrink-0 rounded-sm border border-gray-300"
          :style="swatchStyle(item)"
        />
        <span class="flex-1 truncate">{{ displayName(item.id) }}</span>
        <span v-if="item.count" class="text-gray-400">{{ item.count.toLocaleString() }}</span>
      </button>
    </div>
  </div>
</template>
