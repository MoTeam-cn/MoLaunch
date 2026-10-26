<script setup lang="ts">
/**
 * 皮肤画布
 *
 * 以像素风渲染 64×64 贴图，支持缩放、部件高亮、网格与指针绘制。
 * 只负责"画"和"把指针坐标换算成像素坐标"，绘制逻辑由 useSkinEditor 提供。
 */

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { PixelBuffer } from '@/utils/skin-editor/pixel-buffer'
import { findPartAt, type SkinPart, type SkinPartId } from '@/utils/skin-editor/layout'
import { drawBufferToCanvas } from '@/utils/skin-editor/image'

const props = defineProps<{
  buffer: PixelBuffer
  /** 每次像素变化时递增，用于触发重绘 */
  revision: number
  /** 放大倍数（每像素占多少屏幕像素） */
  zoom: number
  parts: SkinPart[]
  activePart: SkinPartId
}>()

const emit = defineEmits<{
  strokeStart: [x: number, y: number]
  strokeMove: [x: number, y: number]
  strokeEnd: []
  partChange: [part: SkinPartId]
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const drawing = ref(false)

const canvasStyle = computed(() => ({
  width: `${props.buffer.width * props.zoom}px`,
  height: `${props.buffer.height * props.zoom}px`,
}))

function redraw(): void {
  const canvas = canvasRef.value
  if (!canvas) return
  drawBufferToCanvas(canvas, props.buffer, props.zoom)
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // 网格（缩放到能看清像素时才画）
  if (props.zoom >= 6) {
    ctx.strokeStyle = 'rgba(15, 23, 42, 0.08)'
    ctx.lineWidth = 1
    for (let i = 1; i < props.buffer.width; i++) {
      const p = i * props.zoom + 0.5
      ctx.beginPath()
      ctx.moveTo(p, 0)
      ctx.lineTo(p, canvas.height)
      ctx.moveTo(0, p)
      ctx.lineTo(canvas.width, p)
      ctx.stroke()
    }
  }

  // 当前部件高亮
  const part = props.parts.find((item) => item.id === props.activePart)
  if (!part) return
  ctx.fillStyle = 'rgba(19, 112, 243, 0.16)'
  ctx.strokeStyle = 'rgba(19, 112, 243, 0.85)'
  ctx.lineWidth = 2
  for (const rect of [part.base, part.overlay]) {
    if (!rect) continue
    ctx.fillRect(rect.x * props.zoom, rect.y * props.zoom, rect.w * props.zoom, rect.h * props.zoom)
    ctx.strokeRect(
      rect.x * props.zoom + 1,
      rect.y * props.zoom + 1,
      rect.w * props.zoom - 2,
      rect.h * props.zoom - 2,
    )
  }
}

/** 指针坐标 → 皮肤像素坐标 */
function toPixel(event: PointerEvent): { x: number; y: number } | null {
  const canvas = canvasRef.value
  if (!canvas) return null
  const rect = canvas.getBoundingClientRect()
  if (rect.width === 0 || rect.height === 0) return null
  const x = Math.floor(((event.clientX - rect.left) / rect.width) * props.buffer.width)
  const y = Math.floor(((event.clientY - rect.top) / rect.height) * props.buffer.height)
  if (x < 0 || y < 0 || x >= props.buffer.width || y >= props.buffer.height) return null
  return { x, y }
}

function onPointerDown(event: PointerEvent): void {
  const point = toPixel(event)
  if (!point) return
  drawing.value = true
  ;(event.target as HTMLElement | null)?.setPointerCapture?.(event.pointerId)
  const part = findPartAt(props.parts, point.x, point.y)
  if (part && part.id !== props.activePart) emit('partChange', part.id)
  emit('strokeStart', point.x, point.y)
}

function onPointerMove(event: PointerEvent): void {
  if (!drawing.value) return
  const point = toPixel(event)
  if (!point) return
  emit('strokeMove', point.x, point.y)
}

function stopDrawing(): void {
  if (!drawing.value) return
  drawing.value = false
  emit('strokeEnd')
}

watch(() => [props.revision, props.zoom, props.activePart], redraw)

onMounted(redraw)
onBeforeUnmount(() => {
  drawing.value = false
})
</script>

<template>
  <div class="inline-flex flex-col items-center gap-2">
    <div class="rounded border border-gray-300 bg-[repeating-conic-gradient(#f2f3f5_0_25%,#ffffff_0_50%)] bg-[length:16px_16px] p-1">
      <canvas
        ref="canvasRef"
        :style="canvasStyle"
        class="block cursor-crosshair touch-none"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="stopDrawing"
        @pointercancel="stopDrawing"
        @pointerleave="stopDrawing"
      />
    </div>
    <p class="text-xs text-gray-500">按住拖动绘制 · 点击部件可自动切换</p>
  </div>
</template>
