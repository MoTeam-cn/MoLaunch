<script setup lang="ts">
/**
 * 投影切片画布
 *
 * 渲染后端返回的视口切片；支持鼠标框选（返回平面坐标范围）供填充/替换使用。
 * 空气格保持棋盘背景，不绘制颜色。
 */

import { computed, ref, watch } from 'vue'
import { blockColor, type SchematicSlice } from '@/utils/schematic/decode'

const props = defineProps<{
  slice: SchematicSlice
  palette: readonly string[]
  airIndex: number
  /** 每格像素 */
  cellSize: number
  /** 是否允许框选（编辑模式） */
  selectable: boolean
}>()

const emit = defineEmits<{
  hover: [info: { x: number; y: number; id: string } | null]
  select: [range: { x: number; y: number; w: number; h: number }]
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const dragging = ref(false)
const anchor = ref<{ x: number; y: number } | null>(null)
const cursor = ref<{ x: number; y: number } | null>(null)

const pixelWidth = computed(() => props.slice.width * props.cellSize)
const pixelHeight = computed(() => props.slice.height * props.cellSize)

const selectionBox = computed(() => {
  if (!anchor.value || !cursor.value) return null
  const x0 = Math.min(anchor.value.x, cursor.value.x)
  const y0 = Math.min(anchor.value.y, cursor.value.y)
  const w = Math.abs(cursor.value.x - anchor.value.x) + 1
  const h = Math.abs(cursor.value.y - anchor.value.y) + 1
  return { x: x0, y: y0, w, h }
})

function redraw(): void {
  const canvas = canvasRef.value
  if (!canvas) return
  canvas.width = Math.max(pixelWidth.value, 1)
  canvas.height = Math.max(pixelHeight.value, 1)
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const size = props.cellSize
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  for (let row = 0; row < props.slice.height; row++) {
    for (let col = 0; col < props.slice.width; col++) {
      const index = props.slice.cells[row * props.slice.width + col]
      if (index === props.airIndex) continue
      const id = props.palette[index]
      if (!id) continue
      ctx.fillStyle = blockColor(id)
      ctx.fillRect(col * size, row * size, size, size)
    }
  }

  if (size >= 6) {
    ctx.strokeStyle = 'rgba(15, 23, 42, 0.10)'
    ctx.lineWidth = 1
    for (let col = 1; col < props.slice.width; col++) {
      ctx.beginPath()
      ctx.moveTo(col * size + 0.5, 0)
      ctx.lineTo(col * size + 0.5, canvas.height)
      ctx.stroke()
    }
    for (let row = 1; row < props.slice.height; row++) {
      ctx.beginPath()
      ctx.moveTo(0, row * size + 0.5)
      ctx.lineTo(canvas.width, row * size + 0.5)
      ctx.stroke()
    }
  }

  const box = selectionBox.value
  if (box) {
    ctx.fillStyle = 'rgba(19, 112, 243, 0.22)'
    ctx.strokeStyle = 'rgba(19, 112, 243, 0.9)'
    ctx.lineWidth = 2
    ctx.fillRect(box.x * size, box.y * size, box.w * size, box.h * size)
    ctx.strokeRect(box.x * size + 1, box.y * size + 1, box.w * size - 2, box.h * size - 2)
  }
}

function toCell(event: MouseEvent): { x: number; y: number } | null {
  const canvas = canvasRef.value
  if (!canvas) return null
  const rect = canvas.getBoundingClientRect()
  if (rect.width === 0 || rect.height === 0) return null
  const x = Math.floor(((event.clientX - rect.left) / rect.width) * props.slice.width)
  const y = Math.floor(((event.clientY - rect.top) / rect.height) * props.slice.height)
  if (x < 0 || y < 0 || x >= props.slice.width || y >= props.slice.height) return null
  return { x, y }
}

function onDown(event: MouseEvent): void {
  const cell = toCell(event)
  if (!cell) return
  if (!props.selectable) {
    emit('select', { x: cell.x, y: cell.y, w: 1, h: 1 })
    return
  }
  dragging.value = true
  anchor.value = cell
  cursor.value = cell
  redraw()
}

function onMove(event: MouseEvent): void {
  const cell = toCell(event)
  if (!cell) {
    emit('hover', null)
    return
  }
  const index = props.slice.cells[cell.y * props.slice.width + cell.x]
  emit('hover', { x: cell.x, y: cell.y, id: props.palette[index] ?? 'minecraft:air' })
  if (!dragging.value) return
  cursor.value = cell
  redraw()
}

function onUp(): void {
  if (!dragging.value) return
  dragging.value = false
  const box = selectionBox.value
  if (box) emit('select', box)
  anchor.value = null
  cursor.value = null
  redraw()
}

watch(() => [props.slice, props.cellSize], redraw, { immediate: true })
</script>

<template>
  <div
    class="inline-flex rounded border border-gray-300 bg-[repeating-conic-gradient(#f2f3f5_0_25%,#ffffff_0_50%)] bg-[length:12px_12px] p-1"
  >
    <canvas
      ref="canvasRef"
      :style="{ width: pixelWidth + 'px', height: pixelHeight + 'px' }"
      class="block cursor-crosshair"
      @mousedown="onDown"
      @mousemove="onMove"
      @mouseup="onUp"
      @mouseleave="onUp"
    />
  </div>
</template>
