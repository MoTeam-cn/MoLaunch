<script setup lang="ts">
/**
 * 模型 3D 预览（lodestone ThreeStructureRenderer）
 *
 * blockstate/模型 JSON → rp_read_many 批量读取引用链 → 合并原版资源构建
 * Resources → Structure([1,1,1]) 单方块渲染。拖拽旋转，滚轮缩放。
 */
import { onBeforeUnmount, ref, watch } from 'vue'
import * as THREE from 'three'
import { CubeTransparentIcon, CursorArrowRaysIcon } from '@heroicons/vue/24/outline'
import { Structure, ThreeStructureRenderer } from '@mattzh72/lodestone'
import Tooltip from '@/components/common/Tooltip.vue'
import { rpReadMany } from '@/utils/api/tools'
import { buildPreviewResources, type UV4 } from '@/utils/resourcepack/previewResources'

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

interface AnimVertex {
  index: number
  baseUv: [number, number]
  baseLimit: [number, number, number, number]
}
interface AnimGroup {
  mesh: THREE.Mesh
  vertices: AnimVertex[]
}
interface AnimState {
  frames: UV4[]
  frame: number
  groups: AnimGroup[]
}
let animStates: AnimState[] = []
let animClock = 0
let lastLoopTime = 0
/** Minecraft 动画纹理默认速度：每帧 1 tick = 50ms */
const ANIM_INTERVAL_MS = 50

function clamp(v: number, min: number, max: number) {
  return Math.min(max, Math.max(min, v))
}

function disposeRenderer() {
  disposed = true
  animStates = []
  if (rafId) cancelAnimationFrame(rafId)
  rafId = 0
  renderer?.dispose()
  renderer = null
}

/** 扫描 chunk 几何，按 frame0 的 texLimit 匹配动画纹理顶点并快照初始 UV，供帧轮播平移 */
function setupAnimations(animations: Record<string, UV4[]>) {
  animStates = []
  animClock = 0
  const meshes = (renderer as unknown as { chunkMeshes?: THREE.Mesh[] }).chunkMeshes ?? []
  for (const [id, frames] of Object.entries(animations)) {
    if (frames.length < 2) continue
    const base = frames[0]
    const groups: AnimGroup[] = []
    for (const mesh of meshes) {
      const uvAttr = mesh.geometry.getAttribute('uv')
      const limit = mesh.geometry.getAttribute('texLimit')
      if (!uvAttr || !limit) continue
      const vertices: AnimVertex[] = []
      for (let i = 0; i < limit.count; i++) {
        if (
          Math.abs(limit.getX(i) - base[0]) < 1e-6 &&
          Math.abs(limit.getY(i) - base[1]) < 1e-6 &&
          Math.abs(limit.getZ(i) - base[2]) < 1e-6 &&
          Math.abs(limit.getW(i) - base[3]) < 1e-6
        ) {
          vertices.push({
            index: i,
            baseUv: [uvAttr.getX(i), uvAttr.getY(i)],
            baseLimit: [limit.getX(i), limit.getY(i), limit.getZ(i), limit.getW(i)],
          })
        }
      }
      if (vertices.length) groups.push({ mesh, vertices })
    }
    if (groups.length) {
      animStates.push({ frames, frame: 0, groups })
      console.log(
        `[preview] 动画纹理 ${id}：${frames.length} 帧，` +
          `${groups.reduce((n, g) => n + g.vertices.length, 0)} 顶点已接入帧轮播`,
      )
    }
  }
}

/** 将动画纹理顶点 uv/texLimit 平移到第 frameIndex 帧（相对 frame0 快照位移，无累计误差） */
function applyAnimFrame(state: AnimState, frameIndex: number) {
  if (frameIndex === state.frame) return
  const base = state.frames[0]
  const next = state.frames[frameIndex]
  const dU = next[0] - base[0]
  const dV = next[1] - base[1]
  for (const group of state.groups) {
    const uvAttr = group.mesh.geometry.getAttribute('uv')
    const limit = group.mesh.geometry.getAttribute('texLimit')
    for (const v of group.vertices) {
      uvAttr.setXY(v.index, v.baseUv[0] + dU, v.baseUv[1] + dV)
      limit.setXYZW(
        v.index,
        v.baseLimit[0] + dU,
        v.baseLimit[1] + dV,
        v.baseLimit[2] + dU,
        v.baseLimit[3] + dV,
      )
    }
    uvAttr.needsUpdate = true
    limit.needsUpdate = true
  }
  state.frame = frameIndex
}

function startLoop() {
  lastLoopTime = performance.now()
  const loop = () => {
    if (disposed || !renderer) return
    const now = performance.now()
    const elapsed = now - lastLoopTime
    lastLoopTime = now
    if (autoRotate.value && !dragging) yaw += 0.006
    const target = renderer.getCamera().target
    const x = target[0] + radius * Math.cos(pitch) * Math.sin(yaw)
    const y = target[1] + radius * Math.sin(pitch)
    const z = target[2] + radius * Math.cos(pitch) * Math.cos(yaw)
    renderer.setCameraPosition([x, y, z])
    if (animStates.length) {
      animClock += elapsed
      for (const state of animStates) {
        applyAnimFrame(state, Math.floor(animClock / ANIM_INTERVAL_MS) % state.frames.length)
      }
    }
    renderer.drawStructure()
    rafId = requestAnimationFrame(loop)
  }
  rafId = requestAnimationFrame(loop)
}

async function loadPreview() {
  error.value = ''
  loading.value = true
  disposeRenderer()
  disposed = false
  try {
    const res = await rpReadMany(props.workDir, props.relPath)
    if (res.error) throw new Error(res.error)
    const files = new Map(Object.entries(res.files))
    const { resources, blockId, animations } = await buildPreviewResources(files, res.root)
    const canvas = canvasEl.value
    if (!canvas) return
    const structure = new Structure([1, 1, 1])
    structure.addBlock([0, 0, 0], blockId)
    renderer = new ThreeStructureRenderer(canvas, structure, resources)
    const internals = renderer as unknown as {
      opaqueMaterial?: { side: number }
      transparentMaterial?: { side: number }
      chunkMeshes?: unknown[]
    }
    // 物品/模型平面只有单面几何（builtin/generated 仅 south），背面会被 FrontSide 剔除成空白，改为双面
    if (internals.opaqueMaterial) internals.opaqueMaterial.side = THREE.DoubleSide
    if (internals.transparentMaterial) internals.transparentMaterial.side = THREE.DoubleSide
    console.log(
      `[preview] renderer 就绪：chunkMeshes=${internals.chunkMeshes?.length ?? 0}，` +
        `canvas=${canvas.clientWidth || 360}x${canvas.clientHeight || 330}`,
    )
    renderer.setViewport(
      0,
      0,
      canvas.clientWidth || 360,
      canvas.clientHeight || 330,
      window.devicePixelRatio,
    )
    await renderer.whenReady()
    if (renderer) {
      setupAnimations(animations)
      startLoop()
    }
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
      class="relative h-[330px] overflow-hidden rounded border border-gray-200 bg-gray-900"
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
