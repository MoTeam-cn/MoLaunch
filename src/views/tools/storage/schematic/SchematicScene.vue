<script setup lang="ts">
/**
 * 投影 3D 预览
 *
 * 用自研体素渲染器画整个投影：按原版方块模型展开几何，相邻面自动剔除，
 * 台阶是半个、楼梯是阶梯、花草是十字面。体素数据压缩后一次取回。
 * 左键旋转、右键或按住 Shift 平移、滚轮缩放。
 */

import { onBeforeUnmount, ref, watch } from 'vue'
import { ArrowsPointingOutIcon, CubeTransparentIcon, HandRaisedIcon } from '@heroicons/vue/24/outline'
import Tooltip from '@/components/common/Tooltip.vue'
import { schematicVolume } from '@/utils/api/tools/schematic'
import { decodeZlibU16 } from '@/utils/schematic/decode'
import {
  MIN_RADIUS,
  VoxelRenderer,
  clamp,
  loadVoxelPack,
  orbitPosition,
  panTarget,
  type Vec3,
  type VoxelVolume,
} from '@/utils/voxel3d'

const props = defineProps<{
  sessionId: string
  /** 世界尺寸，用于相机初始距离 */
  size: [number, number, number]
  /** 方块数据版本号，变化后重建场景 */
  revision?: number
}>()

const stageEl = ref<HTMLDivElement | null>(null)
const canvasEl = ref<HTMLCanvasElement | null>(null)
const loading = ref(false)
const error = ref('')
const stats = ref('')
const autoRotate = ref(false)

let renderer: VoxelRenderer | null = null
let resizeObserver: ResizeObserver | null = null
let rafId = 0
let disposed = false
let yaw = 0.72
let pitch = 0.42
let radius = 60
let maxRadius = 600
let panning = false
let dragging = false
let lastX = 0
let lastY = 0
let target: Vec3 = [0, 0, 0]

function applyCamera(): void {
  if (!renderer) return
  renderer.setCamera(orbitPosition(target, radius, yaw, pitch), target)
}

function disposeRenderer(): void {
  disposed = true
  if (rafId) cancelAnimationFrame(rafId)
  rafId = 0
  resizeObserver?.disconnect()
  resizeObserver = null
  renderer?.dispose()
  renderer = null
}

function applyViewport(): void {
  const canvas = canvasEl.value
  if (!canvas || !renderer) return
  renderer.setViewport(
    Math.max(canvas.clientWidth, 1),
    Math.max(canvas.clientHeight, 1),
    Math.min(window.devicePixelRatio || 1, 2),
  )
}

/** 自动旋转走独立的渲染循环，开启时才跑 */
function startLoop(): void {
  const loop = () => {
    if (disposed || !renderer) return
    if (autoRotate.value && !dragging) {
      yaw += 0.004
      applyCamera()
    }
    renderer.renderOnce()
    rafId = requestAnimationFrame(loop)
  }
  rafId = requestAnimationFrame(loop)
}

async function loadScene(): Promise<void> {
  error.value = ''
  stats.value = ''
  disposeRenderer()
  disposed = false
  loading.value = true
  try {
    const [loaded, packed] = await Promise.all([loadVoxelPack(), schematicVolume(props.sessionId)])
    if (disposed) return
    const blocks = await decodeZlibU16(packed.blocksBase64)
    if (disposed) return

    const volume: VoxelVolume = {
      blocks,
      palette: packed.palette,
      size: packed.size,
      airIndex: packed.airIndex,
    }

    const [sx, sy, sz] = packed.size
    target = [sx / 2, sy / 2, sz / 2]
    maxRadius = Math.hypot(sx, sy, sz) * 1.6
    radius = Math.max(sx, sy, sz) * 1.15

    const canvas = canvasEl.value
    if (!canvas) return
    renderer = new VoxelRenderer(canvas, loaded.pack, loaded.atlas)
    const faces = renderer.setVolume(volume)
    applyViewport()
    applyCamera()
    stats.value = `方块 ${packed.total.toLocaleString()} · 可见面 ${faces.toLocaleString()}`
    observeResize()
    startLoop()
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

function observeResize(): void {
  resizeObserver?.disconnect()
  const stage = stageEl.value
  if (!stage) return
  resizeObserver = new ResizeObserver(() => {
    applyViewport()
    renderer?.renderOnce()
  })
  resizeObserver.observe(stage)
}

function resetView(): void {
  const [sx, sy, sz] = props.size
  yaw = 0.72
  pitch = 0.42
  radius = Math.max(sx, sy, sz) * 1.15
  target = [sx / 2, sy / 2, sz / 2]
  applyCamera()
}

function onPointerDown(event: PointerEvent): void {
  dragging = true
  panning = event.button === 2 || event.button === 1 || event.shiftKey
  lastX = event.clientX
  lastY = event.clientY
  canvasEl.value?.setPointerCapture(event.pointerId)
}

function onPointerMove(event: PointerEvent): void {
  if (!dragging) return
  const dx = event.clientX - lastX
  const dy = event.clientY - lastY
  lastX = event.clientX
  lastY = event.clientY
  if (panning) {
    target = panTarget(target, radius, yaw, pitch, dx, dy, canvasEl.value?.clientHeight ?? 600)
  } else {
    yaw -= dx * 0.008
    pitch = clamp(pitch + dy * 0.008, -1.45, 1.45)
  }
  applyCamera()
}

function onPointerUp(): void {
  dragging = false
  panning = false
}

function onWheel(event: WheelEvent): void {
  const factor = event.deltaY > 0 ? 1.25 : 0.8
  radius = clamp(radius * factor, MIN_RADIUS, maxRadius)
  applyCamera()
}

watch(() => [props.sessionId, props.revision], loadScene, { immediate: true })
onBeforeUnmount(disposeRenderer)
</script>

<template>
  <section class="flex h-full min-h-[420px] flex-col gap-2 rounded-lg border border-gray-200 bg-white p-3">
    <div class="flex flex-wrap items-center gap-2">
      <CubeTransparentIcon class="h-4 w-4 shrink-0 text-gray-500" />
      <h3 class="text-sm font-medium text-gray-700">3D 预览</h3>
      <Tooltip v-if="stats" :text="stats" class="min-w-0 flex-1 truncate text-xs text-gray-400">
        <span class="w-full truncate">{{ stats }}</span>
      </Tooltip>
      <span class="flex shrink-0 items-center gap-1 text-xs text-gray-400">
        <HandRaisedIcon class="h-3.5 w-3.5" />
        左键旋转 · 右键/Shift 平移 · 滚轮缩放
      </span>
      <span class="ml-auto flex shrink-0 items-center gap-2 text-xs">
        <button
          class="rounded-sm border border-gray-200 px-2 py-1 text-gray-600 hover:border-blue-400 hover:bg-blue-50"
          :class="autoRotate ? 'border-blue-400 bg-blue-50 text-blue-600' : ''"
          @click="autoRotate = !autoRotate"
        >
          自动旋转
        </button>
        <button
          class="rounded-sm border border-gray-200 px-2 py-1 text-gray-600 hover:border-blue-400 hover:bg-blue-50"
          @click="resetView"
        >
          <ArrowsPointingOutIcon class="h-3.5 w-3.5" />
        </button>
      </span>
    </div>
    <div ref="stageEl" class="relative min-h-[360px] flex-1 overflow-hidden rounded-sm bg-slate-900">
      <canvas
        ref="canvasEl"
        class="h-full w-full touch-none select-none cursor-grab active:cursor-grabbing"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
        @contextmenu.prevent
        @wheel.prevent="onWheel"
      ></canvas>
      <div v-if="loading" class="absolute inset-0 grid place-items-center bg-slate-900/80 text-sm text-gray-300">
        正在构建立体场景…
      </div>
      <div
        v-if="error && !loading"
        class="absolute inset-0 flex flex-col items-center justify-center gap-1 bg-slate-900/90 px-4 text-center text-gray-300"
      >
        <p class="text-sm">3D 预览不可用</p>
        <p class="max-w-[80%] break-all text-xs text-gray-400">{{ error }}</p>
      </div>
    </div>
  </section>
</template>
