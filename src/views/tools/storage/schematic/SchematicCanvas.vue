<script setup lang="ts">
/**
 * 投影切片画布
 *
 * 把二维切片按方块配色渲染成像素图；空气格保持棋盘背景色（不绘制）。
 * 鼠标悬停时向上抛出当前格子信息，供页面展示方块名。
 */

import { computed, ref, watch } from 'vue'
import { blockColor, type SchematicSlice } from '@/utils/schematic/decode'

const props = defineProps<{
  slice: SchematicSlice
  palette: readonly string[]
  airIndex: number
  /** 画布最大边长（px） */
  maxSize?: number
}>()

const emit = defineEmits<{
  hover: [info: { x: number; y: number; id: string } | null]
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)

const cellSize = computed(() => {
  const limit = props.maxSize ?? 560
  const longest = Math.max(props.slice.width, props.slice.height, 1)
  return Math.max(1, Math.min(24, Math.floor(limit / longest)))
})

const pixelWidth = computed(() => props.slice.width * cellSize.value)
const pixelHeight = computed(() => props.slice.height * cellSize.value)

function redraw(): void {
  const canvas = canvasRef.value
  if (!canvas) return
  canvas.width = Math.max(pixelWidth.value, 1)
  canvas.height = Math.max(pixelHeight.value, 1)
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  ctx.clearRect(0, 0, canvas.width, canvas.height)
  const size = cellSize.value
  for (let y = 0; y < props.slice.height; y++) {
    for (let x = 0; x < props.slice.width; x++) {
      const index = props.slice.cells[y * props.slice.width + x]
      if (index === props.airIndex) continue
      const id = props.palette[index]
      if (!id) continue
      ctx.fillStyle = blockColor(id)
      ctx.fillRect(x * size, y * size, size, size)
    }
  }

  // 网格：格子够大时才画，避免噪点
  if (size >= 8) {
    ctx.strokeStyle = 'rgba(15, 23, 42, 0.12)'
    ctx.lineWidth = 1
    for (let x = 1; x < props.slice.width; x++) {
      ctx.beginPath()
      ctx.moveTo(x * size + 0.5, 0)
      ctx.lineTo(x * size + 0.5, canvas.height)
      ctx.stroke()
    }
    for (let y = 1; y < props.slice.height; y++) {
      ctx.beginPath()
      ctx.moveTo(0, y * size + 0.5)
      ctx.lineTo(canvas.width, y * size + 0.5)
      ctx.stroke()
    }
  }
}

function onMove(event: MouseEvent): void {
  const canvas = canvasRef.value
  if (!canvas) return
  const rect = canvas.getBoundingClientRect()
  const x = Math.floor(((event.clientX - rect.left) / rect.width) * props.slice.width)
  const y = Math.floor(((event.clientY - rect.top) / rect.height) * props.slice.height)
  if (x < 0 || y < 0 || x >= props.slice.width || y >= props.slice.height) {
    emit('hover', null)
    return
  }
  const index = props.slice.cells[y * props.slice.width + x]
  emit('hover', { x, y, id: props.palette[index] ?? 'minecraft:air' })
}

watch(() => props.slice, redraw, { immediate: true })
watch(cellSize, redraw)
</script>

<template>
  <div
    class="inline-flex rounded border border-gray-300 bg-[repeating-conic-gradient(#f2f3f5_0_25%,#ffffff_0_50%)] bg-[length:12px_12px] p-1"
  >
    <canvas
      ref="canvasRef"
      :style="{ width: pixelWidth + 'px', height: pixelHeight + 'px' }"
      class="block"
      @mousemove="onMove"
      @mouseleave="emit('hover', null)"
    />
  </div>
</template>
