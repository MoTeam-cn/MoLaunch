<script setup lang="ts">
/**
 * 模型 3D 预览（lodestone ThreeStructureRenderer）
 *
 * blockstate/模型 JSON → rp_read_many 批量读取引用链 → 合并原版资源构建
 * Resources → Structure([1,1,1]) 单方块渲染。拖拽旋转，滚轮缩放。
 */
import { onBeforeUnmount, ref, watch } from 'vue'
import { CubeTransparentIcon, CursorArrowRaysIcon } from '@heroicons/vue/24/outline'
import { Structure, ThreeStructureRenderer } from '@mattzh72/lodestone'
import Tooltip from '@/components/common/Tooltip.vue'
import { rpReadMany } from '@/utils/api/tools'
import { buildPreviewResources } from '@/utils/resourcepack/previewResources'

const props = defineProps<{
  workDir: string
  relPath: string
  name: string
}>()

const emit = defineEmits<{ (e: 'failed', message: string): void }>()

const canvasEl = ref<HTMLCanvasElement | null>(null)
const loading = ref(false)
const error = ref('')
const autoRotate = ref(true)

let renderer: ThreeStructureRenderer | null = null
let rafId = 0
let disposed = false
let yaw = 0.6
let pitch = 0.3
let radius = 5
let dragging = false
let lastX = 0
let lastY = 0

function clamp(v: number, min: number, max: number) {
  return Math.min(max, Math.max(min, v))
}

function disposeRenderer() {
  disposed = true
  if (rafId) cancelAnimationFrame(rafId)
  rafId = 0
  renderer?.dispose()
  renderer = null
}

function startLoop() {
  const loop = () => {
    if (disposed || !renderer) return
    if (autoRotate.value && !dragging) yaw += 0.006
    const target = renderer.getCamera().target
    const x = target[0] + radius * Math.cos(pitch) * Math.sin(yaw)
    const y = target[1] + radius * Math.sin(pitch)
    const z = target[2] + radius * Math.cos(pitch) * Math.cos(yaw)
    renderer.setCameraPosition([x, y, z])
    renderer.drawStructure()
    rafId = requestAnimationFrame(loop)
  }
  rafId = requestAnimationFrame(loop)
}

async function loadPreview() {
  error.value = ''
  loading.value = true
  disposeRenderer()
  try {
    const res = await rpReadMany(props.workDir, props.relPath)
    if (res.error) throw new Error(res.error)
    const files = new Map(Object.entries(res.files))
    const { resources, blockId } = await buildPreviewResources(files, res.root)
    const canvas = canvasEl.value
    if (!canvas) return
    const structure = new Structure([1, 1, 1])
    structure.addBlock([0, 0, 0], blockId)
    renderer = new ThreeStructureRenderer(canvas, structure, resources)
    renderer.setViewport(
      0,
      0,
      canvas.clientWidth || 360,
      canvas.clientHeight || 420,
      window.devicePixelRatio,
    )
    await renderer.whenReady()
    if (renderer) startLoop()
  } catch (e) {
    console.error('[preview] 3D 预览加载失败:', e)
    error.value = e instanceof Error ? e.message : String(e)
    emit('failed', error.value)
  } finally {
    loading.value = false
  }
}

function onPointerDown(e: PointerEvent) {
  dragging = true
  lastX = e.clientX
  lastY = e.clientY
  canvasEl.value?.setPointerCapture(e.pointerId)
}

function onPointerMove(e: PointerEvent) {
  if (!dragging) return
  yaw -= (e.clientX - lastX) * 0.01
  pitch = clamp(pitch + (e.clientY - lastY) * 0.01, -1.2, 1.2)
  lastX = e.clientX
  lastY = e.clientY
}

function onPointerUp(e: PointerEvent) {
  dragging = false
  canvasEl.value?.releasePointerCapture(e.pointerId)
}

function onWheel(e: WheelEvent) {
  radius = clamp(radius + e.deltaY * 0.004, 2.2, 16)
}

watch(() => props.relPath, loadPreview, { immediate: true })
onBeforeUnmount(disposeRenderer)
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <CubeTransparentIcon class="h-4 w-4 shrink-0 text-gray-500" />
      <Tooltip :text="name" class="min-w-0 flex-1 truncate" overflow-only>
        <h4 class="w-full truncate text-sm font-medium text-gray-700">{{ name }}</h4>
      </Tooltip>
      <span class="flex shrink-0 items-center gap-1 text-xs text-gray-400">
        <CursorArrowRaysIcon class="h-3.5 w-3.5" />
        拖拽旋转 · 滚轮缩放
      </span>
      <button
        class="ml-auto shrink-0 rounded border border-gray-200 px-2 py-1 text-xs text-gray-600 hover:border-blue-400 hover:bg-blue-50"
        :class="autoRotate ? 'border-blue-400 bg-blue-50 text-blue-600' : ''"
        @click="autoRotate = !autoRotate"
      >
        自动旋转
      </button>
    </div>
    <div
      class="relative h-[420px] overflow-hidden rounded border border-gray-200 bg-gray-900"
    >
      <canvas
        ref="canvasEl"
        class="h-full w-full touch-none cursor-grab active:cursor-grabbing"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
        @wheel.prevent="onWheel"
      ></canvas>
      <div
        v-if="loading"
        class="absolute inset-0 grid place-items-center bg-gray-900/70 text-sm text-gray-300"
      >
        加载模型…
      </div>
      <div
        v-if="error"
        class="absolute inset-0 flex flex-col items-center justify-center gap-1 bg-gray-900/90 text-gray-300"
      >
        <p class="text-sm">模型预览失败</p>
        <p class="max-w-[80%] break-all text-xs text-gray-400">{{ error }}</p>
      </div>
    </div>
  </div>
</template>
