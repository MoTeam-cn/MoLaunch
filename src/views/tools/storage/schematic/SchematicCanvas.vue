<script setup lang="ts">
/**
 * 投影切片画布
 *
 * 渲染后端返回的视口切片，并按交互模式派发事件：
 * browse（只悬停）、pan（拖拽平移）、brush（拖拽涂抹）、rect（框选）、point（形状落点）。
 */

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { blockColor, type SchematicSlice } from '@/utils/schematic/decode'
import { drawTexture, resolveTexture, type BlockAtlas } from '@/utils/schematic/textures'
import type { CanvasInteraction } from './types'

const props = defineProps<{
  slice: SchematicSlice
  palette: readonly string[]
  airIndex: number
  /** 每格像素 */
  cellSize: number
  /** 交互模式 */
  interaction: CanvasInteraction
  /** 画笔笔尖尺寸（格） */
  brushSize?: number
  /** 方块贴图图集，为空时退化纯色 */
  atlas?: BlockAtlas | null
}>()

const emit = defineEmits<{
  hover: [info: { x: number; y: number; id: string } | null]
  select: [range: { x: number; y: number; w: number; h: number }]
  paint: [points: Array<[number, number]>]
  pan: [dx: number, dy: number]
  point: [center: { x: number; y: number }]
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const dragging = ref(false)
const anchor = ref<{ x: number; y: number } | null>(null)
const cursor = ref<{ x: number; y: number } | null>(null)
const lastPan = ref<{ x: number; y: number } | null>(null)
const painted = ref<Set<string>>(new Set())

/** 外层容器给到的可用宽度，用来把格子收进栏宽，避免出现横向滚动条 */
const available = ref(0)
const rootEl = ref<HTMLDivElement | null>(null)
let observer: ResizeObserver | null = null

/** 期望格子大小与实际能放下的取小值 */
const effectiveCell = computed(() => {
  if (available.value <= 0) return props.cellSize
  const fit = Math.floor((available.value - 16) / Math.max(props.slice.width, 1))
  return Math.max(1, Math.min(props.cellSize, fit))
})

const pixelWidth = computed(() => props.slice.width * effectiveCell.value)
const pixelHeight = computed(() => props.slice.height * effectiveCell.value)

const brushSize = computed(() => Math.max(props.brushSize ?? 1, 1))

const selectionBox = computed(() => {
  if (props.interaction !== 'rect') return null
  if (!anchor.value || !cursor.value) return null
  const x0 = Math.min(anchor.value.x, cursor.value.x)
  const y0 = Math.min(anchor.value.y, cursor.value.y)
  return {
    x: x0,
    y: y0,
    w: Math.abs(cursor.value.x - anchor.value.x) + 1,
    h: Math.abs(cursor.value.y - anchor.value.y) + 1,
  }
})

function redraw(): void {
  const canvas = canvasRef.value
  if (!canvas) return
  canvas.width = Math.max(pixelWidth.value, 1)
  canvas.height = Math.max(pixelHeight.value, 1)
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const size = effectiveCell.value
  const atlas = props.atlas ?? null
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.imageSmoothingEnabled = false
  for (let row = 0; row < props.slice.height; row++) {
    for (let col = 0; col < props.slice.width; col++) {
      const index = props.slice.cells[row * props.slice.width + col]
      if (index === props.airIndex) continue
      const id = props.palette[index]
      if (!id) continue
      const texture = resolveTexture(atlas, id)
      if (atlas && texture) {
        drawTexture(ctx, atlas, texture, col * size, row * size, size, size)
      } else {
        ctx.fillStyle = blockColor(id)
        ctx.fillRect(col * size, row * size, size, size)
      }
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

  if (painted.value.size > 0) {
    ctx.fillStyle = 'rgba(19, 112, 243, 0.35)'
    painted.value.forEach((key) => {
      const [col, row] = key.split(':').map(Number)
      ctx.fillRect(col * size, row * size, size, size)
    })
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

/** 把笔尖覆盖的格子记入涂抹集合 */
function stampBrush(cell: { x: number; y: number }): void {
  const half = Math.floor((brushSize.value - 1) / 2)
  for (let dy = 0; dy < brushSize.value; dy++) {
    for (let dx = 0; dx < brushSize.value; dx++) {
      const x = cell.x - half + dx
      const y = cell.y - half + dy
      if (x < 0 || y < 0 || x >= props.slice.width || y >= props.slice.height) continue
      painted.value.add(`${x}:${y}`)
    }
  }
}

function onDown(event: MouseEvent): void {
  const cell = toCell(event)
  if (!cell) return
  if (props.interaction === 'browse') {
    emit('select', { x: cell.x, y: cell.y, w: 1, h: 1 })
    return
  }
  if (props.interaction === 'point') {
    emit('point', cell)
    return
  }
  dragging.value = true
  anchor.value = cell
  cursor.value = cell
  if (props.interaction === 'pan') {
    lastPan.value = cell
  } else if (props.interaction === 'brush') {
    painted.value = new Set()
    stampBrush(cell)
  }
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

  if (props.interaction === 'pan') {
    const last = lastPan.value
    if (!last) return
    const dx = cell.x - last.x
    const dy = cell.y - last.y
    if (dx !== 0 || dy !== 0) {
      lastPan.value = cell
      emit('pan', dx, dy)
    }
    return
  }

  cursor.value = cell
  if (props.interaction === 'brush') stampBrush(cell)
  redraw()
}

function onUp(): void {
  if (!dragging.value) return
  dragging.value = false
  if (props.interaction === 'brush' && painted.value.size > 0) {
    const points: Array<[number, number]> = []
    painted.value.forEach((key) => {
      const [x, y] = key.split(':').map(Number)
      points.push([x, y])
    })
    emit('paint', points)
    painted.value = new Set()
  }
  const box = selectionBox.value
  if (props.interaction === 'rect' && box) emit('select', box)
  anchor.value = null
  cursor.value = null
  lastPan.value = null
  redraw()
}

watch(() => [props.slice, effectiveCell.value, props.brushSize, props.atlas], redraw, { immediate: true })

onMounted(() => {
  const parent = rootEl.value?.parentElement
  if (!parent) return
  observer = new ResizeObserver(() => {
    available.value = parent.clientWidth
  })
  observer.observe(parent)
  available.value = parent.clientWidth
})

onBeforeUnmount(() => {
  observer?.disconnect()
  observer = null
})
</script>

<template>
  <div
    ref="rootEl"
    class="inline-flex max-w-full rounded border border-gray-300 bg-[repeating-conic-gradient(var(--checker-a)_0_25%,var(--checker-b)_0_50%)] bg-[length:12px_12px] p-1"
  >
    <canvas
      ref="canvasRef"
      :style="{ width: pixelWidth + 'px', height: pixelHeight + 'px' }"
      class="block select-none"
      :class="interaction === 'pan' ? 'cursor-grab' : 'cursor-crosshair'"
      @mousedown="onDown"
      @mousemove="onMove"
      @mouseup="onUp"
      @mouseleave="onUp"
    />
  </div>
</template>
