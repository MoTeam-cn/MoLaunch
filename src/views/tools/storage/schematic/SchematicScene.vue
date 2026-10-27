<script setup lang="ts">
/**
 * 投影 3D 预览
 *
 * 用渲染库按原版方块模型绘制整个投影：台阶是半个、楼梯是阶梯、花草是十字面，
 * 形状与游戏内一致。体素数据经 zlib 压缩一次取回，遮挡剔除后再交给渲染器。
 * 拖拽旋转、滚轮缩放，可开关自动旋转。
 */

import { onBeforeUnmount, ref, watch } from 'vue'
import { ThreeStructureRenderer } from '@mattzh72/lodestone'
import { ArrowsPointingOutIcon, CubeTransparentIcon } from '@heroicons/vue/24/outline'
import Tooltip from '@/components/common/Tooltip.vue'
import { schematicVolume } from '@/utils/api/tools/schematic'
import { decodeZlibU16 } from '@/utils/schematic/decode'
import { buildStructure } from '@/utils/schematic/structure'
import { getVanillaBase } from '@/utils/resourcepack/previewResources'

const props = defineProps<{
  sessionId: string
  /** 世界尺寸，用于相机初始距离 */
  size: [number, number, number]
}>()

const canvasEl = ref<HTMLCanvasElement | null>(null)
const loading = ref(false)
const error = ref('')
const stats = ref('')
const autoRotate = ref(false)

let renderer: ThreeStructureRenderer | null = null
let rafId = 0
let disposed = false
let yaw = 0.72
let pitch = 0.42
let radius = 60
let dragging = false
let lastX = 0
let lastY = 0
let target: [number, number, number] = [0, 0, 0]

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}

function disposeRenderer(): void {
  disposed = true
  if (rafId) cancelAnimationFrame(rafId)
  rafId = 0
  renderer?.dispose()
  renderer = null
}

function startLoop(): void {
  const loop = () => {
    if (disposed || !renderer) return
    if (autoRotate.value && !dragging) yaw += 0.004
    const camera = renderer.getCamera()
    target = [camera.target[0], camera.target[1], camera.target[2]]
    const x = target[0] + radius * Math.cos(pitch) * Math.sin(yaw)
    const y = target[1] + radius * Math.sin(pitch)
    const z = target[2] + radius * Math.cos(pitch) * Math.cos(yaw)
    renderer.setCameraPosition([x, y, z])
    renderer.drawStructure()
    rafId = requestAnimationFrame(loop)
  }
  rafId = requestAnimationFrame(loop)
}

function applyViewport(): void {
  const canvas = canvasEl.value
  if (!canvas || !renderer) return
  const width = canvas.clientWidth || 640
  const height = canvas.clientHeight || 420
  renderer.setViewport(0, 0, width, height, window.devicePixelRatio)
}

async function loadScene(): Promise<void> {
  error.value = ''
  stats.value = ''
  disposeRenderer()
  disposed = false
  loading.value = true
  try {
    const [vanilla, packed] = await Promise.all([
      getVanillaBase(),
      schematicVolume(props.sessionId),
    ])
    const blocks = await decodeZlibU16(packed.blocksBase64)
    const built = buildStructure(
      { blocks, palette: packed.palette, size: packed.size, airIndex: packed.airIndex },
      vanilla.resources,
    )
    const [sx, sy, sz] = packed.size
    target = [sx / 2, sy / 2, sz / 2]
    radius = Math.max(sx, sy, sz) * 1.7
    const canvas = canvasEl.value
    if (!canvas) return
    renderer = new ThreeStructureRenderer(canvas, built.structure, vanilla.resources)
    applyViewport()
    await renderer.whenReady()
    if (disposed || !renderer) return
    stats.value = `${built.kept.toLocaleString()} 方块（遮挡剔除后 / 非空气 ${built.total.toLocaleString()}）`
    startLoop()
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

function resetView(): void {
  yaw = 0.72
  pitch = 0.42
  radius = Math.max(props.size[0], props.size[1], props.size[2]) * 1.7
}

function onPointerDown(event: PointerEvent): void {
  dragging = true
  lastX = event.clientX
  lastY = event.clientY
  canvasEl.value?.setPointerCapture(event.pointerId)
}

function onPointerMove(event: PointerEvent): void {
  if (!dragging) return
  yaw -= (event.clientX - lastX) * 0.008
  pitch = clamp(pitch + (event.clientY - lastY) * 0.008, -1.35, 1.35)
  lastX = event.clientX
  lastY = event.clientY
}

function onPointerUp(): void {
  dragging = false
}

function onWheel(event: WheelEvent): void {
  const factor = event.deltaY > 0 ? 1.12 : 0.89
  radius = clamp(radius * factor, 4, Math.max(props.size[0], props.size[1], props.size[2]) * 6)
}

watch(() => props.sessionId, loadScene, { immediate: true })
onBeforeUnmount(disposeRenderer)
</script>

<template>
  <section class="flex flex-col gap-2 rounded-lg border border-gray-200 bg-white p-3">
    <div class="flex items-center gap-2">
      <CubeTransparentIcon class="h-4 w-4 shrink-0 text-gray-500" />
      <h3 class="text-sm font-medium text-gray-700">3D 预览</h3>
      <Tooltip v-if="stats" :text="stats" class="min-w-0 flex-1 truncate text-xs text-gray-400">
        <span class="w-full truncate">{{ stats }}</span>
      </Tooltip>
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
    <div class="relative h-[440px] overflow-hidden rounded-sm border border-gray-200 bg-slate-900">
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
        class="absolute inset-0 grid place-items-center bg-slate-900/80 text-sm text-gray-300"
      >
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
