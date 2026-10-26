<script setup lang="ts">
/**
 * 投影立体预览
 *
 * 用俯视高度图画 2:1 等轴测柱体阵列，展示整体轮廓与高度分布；
 * 当前层为空时仍然能看到建筑在哪里，不必逐层翻页。
 */

import { computed, onMounted, ref, watch } from 'vue'
import { compressOverview, drawIsometric } from '@/utils/schematic/isometric'
import type { SchematicOverview } from '@/utils/schematic/decode'

const props = defineProps<{
  overview: SchematicOverview | null
  palette: readonly string[]
  emptyIndex: number
  /** 世界高度（方块数） */
  worldHeight: number
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const drawn = ref(0)
const expanded = ref(false)

const canvasWidth = computed(() => (expanded.value ? 900 : 520))
const canvasHeight = computed(() => (expanded.value ? 620 : 380))

function render(): void {
  const canvas = canvasRef.value
  if (!canvas || !props.overview) return
  const grid = compressOverview(props.overview)
  drawn.value = drawIsometric(canvas, {
    grid,
    palette: props.palette,
    emptyIndex: props.emptyIndex,
    worldHeight: props.worldHeight,
    canvasWidth: canvasWidth.value,
    canvasHeight: canvasHeight.value,
  })
}

watch(
  () => [props.overview, props.palette, expanded.value],
  render,
  { immediate: true },
)

onMounted(render)
</script>

<template>
  <div ref="containerRef" class="flex flex-col gap-2">
    <div class="flex items-center gap-2">
      <span class="text-xs font-medium text-gray-700">立体预览</span>
      <span class="text-xs text-gray-400">
        俯视高度图 · {{ overview?.step && overview.step > 1 ? `降采样 1/${overview.step}` : '原始精度' }}
        · {{ drawn.toLocaleString() }} 列
      </span>
      <button
        type="button"
        class="ml-auto text-xs text-brand-2 hover:text-brand-3"
        @click="expanded = !expanded"
      >
        {{ expanded ? '收起' : '放大' }}
      </button>
    </div>
    <canvas
      ref="canvasRef"
      class="rounded border border-gray-200 bg-gradient-to-b from-gray-50 to-white"
      :style="{ width: canvasWidth + 'px', height: canvasHeight + 'px' }"
    />
  </div>
</template>
