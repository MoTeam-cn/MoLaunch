<script setup lang="ts">
/**
 * 层密度条
 *
 * 每层非空方块数画成柱状条，直观显示建筑集中在哪些高度；
 * 点击任意位置跳到该层，避免在空层之间盲翻。
 */

import { computed } from 'vue'

const props = defineProps<{
  layerCounts: readonly number[]
  current: number
}>()

const emit = defineEmits<{ pick: [layer: number] }>()

const maxCount = computed(() => props.layerCounts.reduce((max, n) => Math.max(max, n), 1))

const bars = computed(() =>
  props.layerCounts.map((count, layer) => ({
    layer,
    ratio: count / maxCount.value,
    empty: count === 0,
    active: layer === props.current,
  })),
)

function onPick(event: MouseEvent): void {
  const target = event.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()
  if (rect.height === 0) return
  const ratio = (event.clientY - rect.top) / rect.height
  const layer = Math.min(Math.max(Math.floor(ratio * props.layerCounts.length), 0), props.layerCounts.length - 1)
  emit('pick', layer)
}
</script>

<template>
  <div class="flex flex-col items-center gap-1">
    <span class="text-[10px] text-gray-400">{{ layerCounts.length - 1 }}</span>
    <div
      class="relative h-32 w-6 cursor-pointer overflow-hidden rounded-sm border border-gray-200 bg-gray-50"
      @click="onPick"
    >
      <div
        v-for="bar in bars"
        :key="bar.layer"
        class="absolute left-0 w-full"
        :class="bar.active ? 'bg-brand-2' : bar.empty ? 'bg-transparent' : 'bg-brand-2/35'"
        :style="{
          top: (bar.layer / layerCounts.length) * 100 + '%',
          height: Math.max(100 / layerCounts.length, 0.6) + '%',
          width: bar.empty ? '100%' : Math.max(bar.ratio * 100, 8) + '%',
        }"
      />
    </div>
    <span class="text-[10px] text-gray-400">0</span>
  </div>
</template>
