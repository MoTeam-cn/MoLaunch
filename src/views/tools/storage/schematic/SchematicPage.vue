<script setup lang="ts">
/**
 * 投影预览 / 编辑
 *
 * 打开 Sponge / Litematica / 原版结构 / 旧版 MCEdit 投影，按轴逐层浏览。
 * 数据常驻后端会话，前端只取当前视口切片，因此不受整体体积限制。
 * 支持填充、替换、撤销重做、旋转镜像与三格式导出。
 */

import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { pickFile, pickSavePath } from '@/utils/fileDialog'
import { toastError, toastSuccess } from '@/utils/toast'
import {
  schematicClose,
  schematicEdit,
  schematicExport,
  schematicHistory,
  schematicOpen,
  schematicSlice,
  schematicTransform,
  type SchematicOpenResult,
  type SliceAxisParam,
} from '@/utils/api/tools/schematic'
import { decodeBlocks, findAirIndex, type SchematicSlice } from '@/utils/schematic/decode'
import SchematicCanvas from './SchematicCanvas.vue'
import SchematicPalette from './SchematicPalette.vue'
import SchematicStats from './SchematicStats.vue'
import SchematicToolbar from './SchematicToolbar.vue'

/** 视口单边最大格子数 */
const VIEW_SIZE = 128

const session = ref<SchematicOpenResult | null>(null)
const slice = ref<SchematicSlice | null>(null)
const sliceOrigin = ref({ x: 0, y: 0 })
const axis = ref<SliceAxisParam>('y')
const layer = ref(0)
const viewport = ref({ x: 0, y: 0 })
const cellSize = ref(4)
const tool = ref<'browse' | 'fill' | 'replace'>('browse')
const selectedBlock = ref(0)
const replaceFrom = ref(0)
const airIndex = ref(0)
const canUndo = ref(false)
const canRedo = ref(false)
const nonAir = ref(0)
const hover = ref<{ x: number; y: number; id: string } | null>(null)
const loading = ref(false)
const busy = ref(false)
const exportFormat = ref('sponge_v2')

const FORMAT_EXT: Record<string, string> = {
  sponge_v2: 'schem',
  litematic: 'litematic',
  structure: 'nbt',
}

const plane = computed(() => {
  if (!session.value) return { width: 1, height: 1, layers: 1 }
  const [sx, sy, sz] = session.value.size
  if (axis.value === 'x') return { width: sz, height: sy, layers: sx }
  if (axis.value === 'z') return { width: sx, height: sy, layers: sz }
  return { width: sx, height: sz, layers: sy }
})

const paletteCounts = computed<Record<string, number>>(() => {
  const map: Record<string, number> = {}
  session.value?.counts.forEach((item) => {
    map[item.id] = item.count
  })
  return map
})

const hoverLabel = computed(() => {
  if (!hover.value) return ''
  return `(${hover.value.x}, ${hover.value.y}) ${hover.value.id.replace('minecraft:', '')}`
})

async function refreshSlice(): Promise<void> {
  if (!session.value) return
  const w = Math.min(VIEW_SIZE, plane.value.width)
  const h = Math.min(VIEW_SIZE, plane.value.height)
  viewport.value = {
    x: Math.min(Math.max(viewport.value.x, 0), Math.max(plane.value.width - w, 0)),
    y: Math.min(Math.max(viewport.value.y, 0), Math.max(plane.value.height - h, 0)),
  }
  const result = await schematicSlice(
    session.value.sessionId,
    axis.value,
    layer.value,
    viewport.value.x,
    viewport.value.y,
    w,
    h,
  )
  slice.value = { width: result.width, height: result.height, cells: decodeBlocks(result.cellsBase64) }
  sliceOrigin.value = { x: result.x, y: result.y }
}

function centerViewport(): void {
  const w = Math.min(VIEW_SIZE, plane.value.width)
  const h = Math.min(VIEW_SIZE, plane.value.height)
  viewport.value = {
    x: Math.max(Math.floor((plane.value.width - w) / 2), 0),
    y: Math.max(Math.floor((plane.value.height - h) / 2), 0),
  }
}

async function openFile(): Promise<void> {
  loading.value = true
  try {
    const path = await pickFile({
      title: '选择投影文件',
      filters: [{ name: '投影文件', extensions: ['schem', 'schematic', 'litematic', 'nbt'] }],
    })
    if (!path) return
    if (session.value) await schematicClose(session.value.sessionId)
    const result = await schematicOpen(path)
    session.value = result
    airIndex.value = findAirIndex(result.palette)
    const firstSolid = result.palette.findIndex((id) => id !== 'minecraft:air')
    selectedBlock.value = firstSolid >= 0 ? firstSolid : 0
    replaceFrom.value = airIndex.value >= 0 ? airIndex.value : 0
    nonAir.value = result.nonAir
    canUndo.value = false
    canRedo.value = false
    layer.value = Math.floor(plane.value.layers / 2)
    centerViewport()
    await refreshSlice()
    toastSuccess(`已打开 ${result.name}（${result.format}）`)
  } catch (err) {
    toastError(`打开失败：${err}`)
  } finally {
    loading.value = false
  }
}

async function applySelection(range: { x: number; y: number; w: number; h: number }): Promise<void> {
  if (!session.value || tool.value === 'browse') return
  busy.value = true
  try {
    const result = await schematicEdit({
      sessionId: session.value.sessionId,
      axis: axis.value,
      layer: layer.value,
      x: sliceOrigin.value.x + range.x,
      y: sliceOrigin.value.y + range.y,
      w: range.w,
      h: range.h,
      block: selectedBlock.value,
      mode: tool.value === 'replace' ? 'replace' : 'fill',
      from: tool.value === 'replace' ? replaceFrom.value : null,
    })
    nonAir.value = result.nonAir
    canUndo.value = result.canUndo
    canRedo.value = result.canRedo
    await refreshSlice()
  } catch (err) {
    toastError(`编辑失败：${err}`)
  } finally {
    busy.value = false
  }
}

async function step(action: 'undo' | 'redo'): Promise<void> {
  if (!session.value) return
  try {
    const result = await schematicHistory(session.value.sessionId, action)
    nonAir.value = result.nonAir
    canUndo.value = result.canUndo
    canRedo.value = result.canRedo
    await refreshSlice()
  } catch (err) {
    toastError(`操作失败：${err}`)
  }
}

async function doTransform(
  kind: 'rotate_cw' | 'rotate_ccw' | 'mirror_x' | 'mirror_z' | 'flip_y',
): Promise<void> {
  if (!session.value) return
  busy.value = true
  try {
    const result = await schematicTransform(session.value.sessionId, kind)
    session.value = { ...session.value, size: result.size }
    nonAir.value = result.nonAir
    canUndo.value = false
    canRedo.value = false
    layer.value = Math.floor(plane.value.layers / 2)
    centerViewport()
    await refreshSlice()
  } catch (err) {
    toastError(`变换失败：${err}`)
  } finally {
    busy.value = false
  }
}

async function doExport(): Promise<void> {
  if (!session.value) return
  busy.value = true
  try {
    const ext = FORMAT_EXT[exportFormat.value] ?? 'schem'
    const target = await pickSavePath({
      title: '导出投影',
      defaultPath: `${session.value.name}.${ext}`,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    })
    if (!target) return
    const result = await schematicExport(session.value.sessionId, target, exportFormat.value)
    toastSuccess(`已导出（${(result.bytes / 1024).toFixed(1)} KB）`)
  } catch (err) {
    toastError(`导出失败：${err}`)
  } finally {
    busy.value = false
  }
}

/** 平移视口（每次移动视口边长的一半） */
async function pan(dx: number, dy: number): Promise<void> {
  if (!session.value) return
  const step = Math.max(Math.floor(VIEW_SIZE / 2), 1)
  viewport.value = { x: viewport.value.x + dx * step, y: viewport.value.y + dy * step }
  await refreshSlice()
}

function zoom(delta: number): void {
  cellSize.value = Math.min(16, Math.max(1, cellSize.value + delta))
}

watch(axis, () => {
  layer.value = Math.floor(plane.value.layers / 2)
  centerViewport()
  void refreshSlice()
})
watch(layer, () => void refreshSlice())

onBeforeUnmount(() => {
  if (session.value) void schematicClose(session.value.sessionId)
})
</script>

<template>
  <section class="flex flex-col gap-4 rounded-lg border border-gray-300 bg-white p-4">
    <SchematicToolbar
      :loading="loading"
      :busy="busy"
      :session="session"
      :axis="axis"
      :layer="layer"
      :plane-layers="plane.layers"
      :tool="tool"
      :can-undo="canUndo"
      :can-redo="canRedo"
      :export-format="exportFormat"
      :non-air="nonAir"
      @open="openFile"
      @update:axis="axis = $event"
      @update:layer="layer = $event"
      @update:tool="tool = $event"
      @update:export-format="exportFormat = $event"
      @undo="step('undo')"
      @redo="step('redo')"
      @transform="doTransform"
      @zoom="zoom"
      @pan="pan"
      @export="doExport"
    />

    <p v-if="!session" class="text-xs text-gray-500">
      支持 Sponge（.schem / .schematic，v1 与 v2）、Litematica（.litematic）、原版结构（.nbt）
      与旧版 MCEdit 投影。打开后按视口分片加载，大投影也能流畅浏览。
    </p>

    <div v-else class="flex flex-wrap items-start gap-4">
      <div class="flex flex-col gap-2">
        <SchematicCanvas
          v-if="slice"
          :slice="slice"
          :palette="session.palette"
          :air-index="airIndex"
          :cell-size="cellSize"
          :selectable="tool !== 'browse'"
          @hover="hover = $event"
          @select="applySelection"
        />
        <span class="h-4 text-xs text-gray-500">{{ hoverLabel }}</span>
      </div>

      <div class="flex min-w-[260px] flex-1 flex-col gap-3">
        <SchematicPalette
          v-if="tool !== 'browse'"
          v-model="selectedBlock"
          :palette="session.palette"
          :counts="paletteCounts"
        />
        <SchematicPalette
          v-if="tool === 'replace'"
          v-model="replaceFrom"
          :palette="session.palette"
          :counts="paletteCounts"
        />
        <SchematicStats :counts="session.counts" :non-air="nonAir" :total="session.total" />
      </div>
    </div>
  </section>
</template>
