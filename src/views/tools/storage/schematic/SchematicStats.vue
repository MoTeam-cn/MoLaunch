<script setup lang="ts">
/**
 * 投影方块统计
 *
 * 展示总方块数、非空气数与按数量降序的方块列表（带配色块与占比）。
 */

import { computed } from 'vue'
import { blockColor } from '@/utils/schematic/decode'
import type { SchematicBlockCount } from '@/utils/api/tools/schematic'

const props = defineProps<{
  counts: SchematicBlockCount[]
  nonAir: number
  total: number
}>()

const rows = computed(() =>
  props.counts.map((item) => ({
    id: item.id,
    count: item.count,
    color: blockColor(item.id),
    percent: props.nonAir > 0 ? (item.count / props.nonAir) * 100 : 0,
  })),
)
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-baseline gap-3">
      <span class="text-sm font-semibold text-gray-900">{{ nonAir.toLocaleString() }}</span>
      <span class="text-xs text-gray-500">非空气方块 / 总体积 {{ total.toLocaleString() }}</span>
    </div>

    <div class="max-h-72 overflow-y-auto rounded border border-gray-200">
      <div
        v-for="row in rows"
        :key="row.id"
        class="flex items-center gap-2 border-b border-gray-100 px-2 py-1 text-xs last:border-b-0"
      >
        <span class="h-3 w-3 shrink-0 rounded-sm border border-gray-300" :style="{ backgroundColor: row.color }" />
        <span class="flex-1 truncate text-gray-700" :title="row.id">{{ row.id.replace('minecraft:', '') }}</span>
        <span class="tabular-nums text-gray-500">{{ row.count.toLocaleString() }}</span>
        <span class="w-12 text-right tabular-nums text-gray-400">{{ row.percent.toFixed(1) }}%</span>
      </div>
      <p v-if="rows.length === 0" class="px-2 py-3 text-center text-xs text-gray-400">没有可统计的方块</p>
    </div>
  </div>
</template>
