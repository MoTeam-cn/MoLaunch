<script setup lang="ts">
/**
 * 投影 3D 预览
 *
 * 按原版方块模型绘制整个投影：台阶是半个、楼梯是阶梯、花草是十字面。
 * 体素数据压缩后一次取回，遮挡剔除再交给渲染器。
 * 左键旋转、右键或按住 Shift 平移、滚轮缩放（向光标位置推进）。
 */

import { onBeforeUnmount, ref, watch } from 'vue'
import * as THREE from 'three'
import { ThreeStructureRenderer } from '@mattzh72/lodestone'
import { ArrowsPointingOutIcon, CubeTransparentIcon, HandRaisedIcon } from '@heroicons/vue/24/outline'
import Tooltip from '@/components/common/Tooltip.vue'
import { schematicVolume } from '@/utils/api/tools/schematic'
import { decodeZlibU16 } from '@/utils/schematic/decode'
import { buildStructure } from '@/utils/schematic/structure'
import { getVanillaBase } from '@/utils/resourcepack/previewResources'

const props = defineProps<{
  sessionId: string
  /** 世界尺寸，用于相机初始距离 */
  size: [number, number, number]
  /** 方块数据版本号，变化后重建场景 */
  revision?: number
}>()

const FOV = 70
const MIN_RADIUS = 2

const stageEl = ref<HTMLDivElement | null>(null)
const canvasEl = ref<HTMLCanvasElement | null>(null)
const loading = ref(false)
const error = ref('')
const stats = ref('')
const autoRotate = ref(false)

let renderer: ThreeStructureRenderer | null = null
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
let target: [number, number, number] = [0, 0, 0]

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}

/** 相机当前所在位置（绕 target 的球面坐标） */
function orbitPosition(): [number, number, number] {
  const cosPitch = Math.cos(pitch)
  return [
    target[0] + radius * cosPitch * Math.sin(yaw),
    target[1] + radius * Math.sin(pitch),
    target[2] + radius * cosPitch * Math.cos(yaw),
  ]
}

/** 近远裁剪面跟着观察距离走：固定 0.1/2000 在几百格的距离上深度精度不够，会互相啃面 */
function syncClipPlanes(): void {
  const camera = (renderer as unknown as { camera?: THREE.PerspectiveCamera } | null)?.camera
  if (!camera) return
  camera.near = Math.max(0.05, radius / 400)
  camera.far = radius * 12 + 512
  camera.updateProjectionMatrix()
}

function applyCamera(): void {
  if (!renderer) return
  syncClipPlanes()
  renderer.setCameraTarget(target)
  renderer.setCameraPosition(orbitPosition())
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
  const width = Math.max(canvas.clientWidth, 1)
  const height = Math.max(canvas.clientHeight, 1)
  renderer.setViewport(0, 0, width, height, window.devicePixelRatio)
}

function startLoop(): void {
  const loop = () => {
    if (disposed || !renderer) return
    if (autoRotate.value && !dragging) {
      yaw += 0.004
      applyCamera()
    }
    renderer.drawStructure()
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
    maxRadius = Math.hypot(sx, sy, sz) * 1.6
    radius = Math.max(sx, sy, sz) * 1.15
    const canvas = canvasEl.value
    if (!canvas) return
    renderer = new ThreeStructureRenderer(canvas, built.structure, vanilla.resources, {
      antialias: true,
      // 阴影贴图默认只覆盖 100×100，投影动辄上百格，整片结构落在范围外会满屏斑驳
      sunlight: { shadow: { enabled: false }, fog: { density: 0 } },
    })
    applyViewport()
    applyCamera()
    fixAtlasSampling()
    await renderer.whenReady()
    if (disposed || !renderer) return
    stats.value = `显示 ${built.kept.toLocaleString()} 方块（遮挡剔除后 / 非空气 ${built.total.toLocaleString()}）`
    observeResize()
    startLoop()
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

/**
 * 图集里每个方块只占 16×16，开着 mipmap 时远处会采样到相邻方块的像素，
 * 大片同种方块的地面就会浮出一层规则条纹。像素风本来就该用最近邻采样。
 */
function fixAtlasSampling(): void {
  const texture = (renderer as unknown as { atlasTexture?: THREE.Texture } | null)?.atlasTexture
  if (!texture) return
  texture.magFilter = THREE.NearestFilter
  texture.minFilter = THREE.NearestFilter
  texture.generateMipmaps = false
  texture.needsUpdate = true
}

function observeResize(): void {
  resizeObserver?.disconnect()
  const stage = stageEl.value
  if (!stage) return
  resizeObserver = new ResizeObserver(() => {
    applyViewport()
    if (renderer) renderer.drawStructure()
  });
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
    // 屏幕位移换算成世界位移：把画布高度对应的可视高度取出来，再乘方向基向量
    const canvas = canvasEl.value
    const viewHeight = 2 * radius * Math.tan((FOV * Math.PI) / 360)
    const scale = viewHeight / Math.max(canvas?.clientHeight ?? 600, 1)
    const cosYaw = Math.cos(yaw)
    const sinYaw = Math.sin(yaw)
    const sinPitch = Math.sin(pitch)
    const cosPitch = Math.cos(pitch)
    // right 与 up 是相机基底在世界的向量
    const rightX = cosYaw;
    const rightZ = -sinYaw;
    const upX = -sinPitch * sinYaw;
    const upY = cosPitch;
    const upZ = -sinPitch * cosYaw;
    target = [
      target[0] - rightX * dx * scale + upX * dy * scale,
      target[1] + upY * dy * scale,
      target[2] - rightZ * dx * scale + upZ * dy * scale,
    ];
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
