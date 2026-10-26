<script setup lang="ts">
/**
 * 投影侧栏
 *
 * 左侧：方块选择器（编辑时显示，替换模式下多一个来源选择）
 * 右侧：方块统计。纯展示组件。
 */

import type { SchematicBlockCount } from '@/utils/api/tools/schematic'
import type { BlockAtlas } from '@/utils/schematic/textures'
import SchematicPalette from './SchematicPalette.vue'
import SchematicStats from './SchematicStats.vue'

defineProps<{
  palette: readonly string[]
  counts: Record<string, number>
  topCounts: SchematicBlockCount[]
  nonAir: number
  total: number
  showPicker: boolean
  showFrom: boolean
  /** 方块贴图图集，为空时选择器只显示纯色 */
  atlas?: BlockAtlas | null
}>()

const selectedBlock = defineModel<number>('selectedBlock', { required: true })
const replaceFrom = defineModel<number>('replaceFrom', { required: true })
</script>

<template>
  <div class="flex min-w-[260px] flex-1 flex-col gap-3">
    <template v-if="showPicker">
      <span class="text-xs text-gray-600">目标方块</span>
      <SchematicPalette
        v-model="selectedBlock"
        :palette="palette"
        :counts="counts"
        :atlas="atlas ?? null"
      />
    </template>
    <template v-if="showFrom">
      <span class="text-xs text-gray-600">来源方块（只替换这一种）</span>
      <SchematicPalette
        v-model="replaceFrom"
        :palette="palette"
        :counts="counts"
        :atlas="atlas ?? null"
      />
    </template>
    <SchematicStats :counts="topCounts" :non-air="nonAir" :total="total" :atlas="atlas ?? null" />
  </div>
</template>
