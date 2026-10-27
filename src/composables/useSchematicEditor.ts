/**
 * 投影编辑器状态与操作
 *
 * 把打开/切片/编辑/撤销/变换/导出全部收在这里，页面只负责布局。
 * 视口按需分片获取，方块实体由后端随会话保留。
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
  schematicOverview,
  schematicSlice,
  schematicTransform,
  type SchematicOpenResult,
  type SliceAxisParam,
} from '@/utils/api/tools/schematic'
import {
  decodeBlocks,
  decodeU16,
  densestLayer,
  findAirIndex,
  surfaceLayer,
  nearestOccupiedLayer,
  type SchematicOverview,
  type SchematicSlice,
} from '@/utils/schematic/decode'
import { loadBlockAtlas, type BlockAtlas } from '@/utils/schematic/textures'
import { EDIT_TOOLS, TOOL_INTERACTION, type SchematicTool } from '@/views/tools/storage/schematic/types'

/** 视口单边最大格子数 */
const VIEW_SIZE = 192

const FORMAT_EXT: Record<string, string> = {
  sponge_v2: 'schem',
  litematic: 'litematic',
  structure: 'nbt',
}

export function useSchematicEditor() {
  const session = ref<SchematicOpenResult | null>(null)
  const slice = ref<SchematicSlice | null>(null)
  const sliceOrigin = ref({ x: 0, y: 0 })
  const axis = ref<SliceAxisParam>('y')
  const layer = ref(0)
  const viewport = ref({ x: 0, y: 0 })
  const cellSize = ref(4)
  const tool = ref<SchematicTool>('browse')
  const selectedBlock = ref(0)
  const replaceFrom = ref(0)
  const airIndex = ref(0)
  const brushSize = ref(3)
  const radius = ref(4)
  const cylinderHeight = ref(6)
  const canUndo = ref(false)
  const canRedo = ref(false)
  const nonAir = ref(0)
  const hover = ref<{ x: number; y: number; id: string } | null>(null)
  const loading = ref(false)
  const busy = ref(false)
  const exportFormat = ref('sponge_v2')
  const overview = ref<SchematicOverview | null>(null)
/** 方块贴图图集：加载失败时为 null，画布退回纯色 */
const atlas = ref<BlockAtlas | null>(null)
  /** 方块数据版本号：编辑、撤销重做、旋转镜像后自增，供 3D 场景知道该重建 */
  const revision = ref(0)
loadBlockAtlas().then((loaded) => {
  atlas.value = loaded
})

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

  const interaction = computed(() => TOOL_INTERACTION[tool.value])
  const needsBlock = computed(() => EDIT_TOOLS.includes(tool.value))
  const hoverLabel = computed(() => {
    if (!hover.value) return ''
    return `(${hover.value.x}, ${hover.value.y}) ${hover.value.id.replace('minecraft:', '')}`
  })

  let refreshTimer: ReturnType<typeof setTimeout> | null = null

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
    slice.value = {
      width: result.width,
      height: result.height,
      cells: decodeBlocks(result.cellsBase64),
    }
    sliceOrigin.value = { x: result.x, y: result.y }
  }

  /** 拉取层密度与俯视高度图（供立体预览与非空层定位） */
  async function loadOverview(): Promise<void> {
    if (!session.value) return
    try {
      const result = await schematicOverview(session.value.sessionId)
      overview.value = {
        width: result.width,
        height: result.height,
        step: result.step,
        topIndex: decodeU16(result.topIndexBase64),
        topHeight: decodeU16(result.topHeightBase64),
        layerCounts: result.layerCounts,
        emptyIndex: result.emptyIndex,
      }
    } catch {
      // 概览是增强项，失败不影响逐层浏览
      overview.value = null
    }
  }

  /** 拖拽平移时合并刷新，避免每格触发一次 IPC */
  function scheduleRefresh(delay = 90): void {
    if (refreshTimer) clearTimeout(refreshTimer)
    refreshTimer = setTimeout(() => {
      refreshTimer = null
      void refreshSlice()
    }, delay)
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
      tool.value = 'browse'
      await loadOverview()
      // 打开后落在地面层：带地形的投影最密层在地底，取密度过半的最上层才看得到建筑
      const dense = overview.value ? surfaceLayer(overview.value.topHeight) : -1
      layer.value = dense >= 0 ? Math.min(dense, plane.value.layers - 1) : Math.floor(plane.value.layers / 2)
      centerViewport()
      await refreshSlice()
      toastSuccess(`已打开 ${result.name}（${result.format}）`)
    } catch (err) {
      toastError(`打开失败：${err}`)
    } finally {
      loading.value = false
    }
  }

  /** 统一编辑入口：按当前工具决定形状 */
  async function applyEdit(params: {
    shape: 'rect' | 'sphere' | 'cylinder' | 'points'
    x: number
    y: number
    w: number
    h: number
    points?: Array<[number, number]>
  }): Promise<void> {
    if (!session.value || !needsBlock.value) return
    busy.value = true
    try {
      const result = await schematicEdit({
        sessionId: session.value.sessionId,
        axis: axis.value,
        layer: layer.value,
        x: params.x,
        y: params.y,
        w: params.w,
        h: params.h,
        block: selectedBlock.value,
        mode: tool.value === 'replace' ? 'replace' : 'fill',
        from: tool.value === 'replace' ? replaceFrom.value : null,
        shape: params.shape,
        radius: radius.value,
        height: cylinderHeight.value,
        points: params.points ?? [],
      })
      nonAir.value = result.nonAir
      revision.value += 1
      canUndo.value = result.canUndo
      canRedo.value = result.canRedo
      await refreshSlice()
    } catch (err) {
      toastError(`编辑失败：${err}`)
    } finally {
      busy.value = false
    }
  }

  function onSelect(range: { x: number; y: number; w: number; h: number }): void {
    void applyEdit({
      shape: 'rect',
      x: sliceOrigin.value.x + range.x,
      y: sliceOrigin.value.y + range.y,
      w: range.w,
      h: range.h,
    })
  }

  function onPaint(points: Array<[number, number]>): void {
    if (points.length === 0) return
    const shifted: Array<[number, number]> = points.map(([x, y]) => [
      x + sliceOrigin.value.x,
      y + sliceOrigin.value.y,
    ])
    void applyEdit({ shape: 'points', x: 0, y: 0, w: 0, h: 0, points: shifted })
  }

  function onPoint(center: { x: number; y: number }): void {
    const shape = tool.value === 'sphere' ? 'sphere' : 'cylinder'
    void applyEdit({
      shape,
      x: center.x + sliceOrigin.value.x,
      y: center.y + sliceOrigin.value.y,
      w: 1,
      h: 1,
    })
  }

  function onPan(dx: number, dy: number): void {
    viewport.value = { x: viewport.value.x + dx, y: viewport.value.y + dy }
    scheduleRefresh()
  }

  async function step(action: 'undo' | 'redo'): Promise<void> {
    if (!session.value) return
    try {
      const result = await schematicHistory(session.value.sessionId, action)
      nonAir.value = result.nonAir
      revision.value += 1
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
      revision.value += 1
      canUndo.value = false
      canRedo.value = false
      await loadOverview()
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

  /** 当前层是否一个方块都没有 */
  const layerEmpty = computed(() => {
    if (!overview.value) return false
    return (overview.value.layerCounts[layer.value] ?? 0) === 0
  })

  /** 跳到最近的非空层 */
  async function jumpToNearestOccupied(): Promise<void> {
    if (!overview.value) return
    const target = nearestOccupiedLayer(overview.value.layerCounts, layer.value)
    if (target >= 0 && target !== layer.value) {
      layer.value = target
    }
  }

  /** 跳到建筑最密集的层 */
  async function jumpToDensest(): Promise<void> {
    if (!overview.value) return
    const target = densestLayer(overview.value.layerCounts)
    if (target >= 0) layer.value = target
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
    if (refreshTimer) clearTimeout(refreshTimer)
    if (session.value) void schematicClose(session.value.sessionId)
  })

  return {
    session,
    slice,
    axis,
    layer,
    cellSize,
    tool,
    selectedBlock,
    replaceFrom,
    airIndex,
    brushSize,
    radius,
    cylinderHeight,
    canUndo,
    canRedo,
    nonAir,
    hover,
    loading,
    busy,
    exportFormat,
    overview,
    plane,
    paletteCounts,
    interaction,
    needsBlock,
    hoverLabel,
    layerEmpty,
    openFile,
    onSelect,
    onPaint,
    onPoint,
    onPan,
    step,
    doTransform,
    doExport,
    jumpToNearestOccupied,
    jumpToDensest,
    atlas,
    revision,
    zoom,
  }
}
