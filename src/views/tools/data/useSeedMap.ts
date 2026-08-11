/**
 * 种子地图组合式逻辑（OpenLayers + WASM Worker）
 *
 * 自定义 'mc' 投影（1 单位=1 方块，extent ±3e7）；群系 DataTile + 结构/出生点/要塞
 * Vector 图层；OL 内置交互 + pointermove/singleclick 命中检测。
 * 职责拆分至 ./useSeedMap/（tileLoader / structureManager / map-init / map-events / config）。
 */
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { toastError, toastSuccess } from '@/utils/toast'
import { getStructIcon, getStructIconUrl } from '@/utils/seedmap/constants'
import { WorkerPool } from '@/utils/seedmap/workerPool'
import { resUrl } from '@/utils/wasm-loader'
import { formatCoord, copyToClipboard } from '@/utils/seedmap/format'
import { getStructuresForVersion } from '@/utils/seedmap/structures'
import type { Dimension, WorkerStructure } from '@/utils/seedmap/types'
import type OlMap from 'ol/Map'
import type DataTile from 'ol/source/DataTile'
import { type Loader as DataTileLoader } from 'ol/source/DataTile'
import type TileLayer from 'ol/layer/Tile'
import type VectorLayer from 'ol/layer/Vector'
import type VectorSource from 'ol/source/Vector'
import type Feature from 'ol/Feature'
import type Overlay from 'ol/Overlay'
import { MIN_ZOOM, MAX_ZOOM, SEEDMAP_MC_VERSIONS, STRUCT_MIN_ZOOM } from './useSeedMap/config'
// re-export mapMcVersionToCubiomes 保持调用方路径兼容
export { mapMcVersionToCubiomes } from './useSeedMap/config'
import { createTileLoader } from './useSeedMap/tileLoader'
import { createStructureManager } from './useSeedMap/structureManager'
import { initSeedMap } from './useSeedMap/map-init'
import { createMapEvents } from './useSeedMap/map-events'

export function useSeedMap() {
  // ===== 控件状态 =====
  const seedInput = ref<string>('')
  const mcVersion = ref<number>(34)  // 默认最新版（26.2 = cubiomes MC_26_2 = MC_NEWEST = 34）
  const dimension = ref<Dimension>(0)
  const largeBiomes = ref<boolean>(false)
  const userX = ref<string>('')
  const userZ = ref<string>('')
  const showCoordPanel = ref<boolean>(false)
  /** 地下群系查看的 Y 坐标（默认 64=海平面） */
  const yCoord = ref<number>(64)
  /** 是否绘制等高线 */
  const doContour = ref<boolean>(false)
  /** 最大渲染高度（0 表示不限制） */
  const ymaxLimit = ref<number>(0)

  /** 默认种子（页面加载时自动使用，但不填入输入框） */
  const DEFAULT_SEED = '12345'

  const versionOptions = SEEDMAP_MC_VERSIONS.map(v => ({ label: v.label, value: v.value }))
  const dimensionOptions = [{ label: '主世界', value: 0 }, { label: '下界', value: -1 }, { label: '末地', value: 1 }]

  // ===== 视图状态 =====
  const loading = ref<boolean>(false)
  const structures = ref<WorkerStructure[]>([])
  const hoverStruct = ref<WorkerStructure | null>(null)
  /** 悬停 spawn/stronghold 时的提示数据（struct 走 hoverStruct） */
  const hoverMarker = ref<{ label: string; x: number; z: number } | null>(null)
  const mouseBlock = ref<{ x: number; z: number } | null>(null)
  const lastClickBlock = ref<{ x: number; z: number } | null>(null)
  /** popup 浮窗数据（含 OL 坐标用于 Overlay 定位） */
  const popupData = ref<{ struct: WorkerStructure; coord: [number, number] } | null>(null)
  /** 鼠标悬停的群系名（异步查询，debounce 300ms） */
  const mouseBiomeName = ref<string>('')
  const showSpawn = ref<boolean>(true)
  const showStronghold = ref<boolean>(true)
  // 默认只勾选村庄，避免全部勾选时地图标记过密
  const selectedStructureTypes = ref<Set<string>>(new Set(['Village']))
  const showNonViable = ref<boolean>(false)

  // ===== OL 实例（非响应式，用 let） =====
  const mapContainer = ref<HTMLDivElement | null>(null)
  const popupContainer = ref<HTMLDivElement | null>(null)
  /** 结构高亮 Feature（style 闭包与事件处理共享） */
  const highlight: { hoverFeat: Feature | null; clickFeat: Feature | null } = { hoverFeat: null, clickFeat: null }
  let map: OlMap | null = null
  let biomeSource: DataTile | null = null
  let biomeLayer: TileLayer | null = null
  let structSource: VectorSource | null = null
  let spawnSource: VectorSource | null = null
  let strongholdSource: VectorSource | null = null
  let clickMarkerSource: VectorSource | null = null
  let spawnLayer: TileLayer | VectorLayer | null = null
  let strongholdLayer: TileLayer | VectorLayer | null = null
  let popupOverlay: Overlay | null = null
  let structLayerRef: VectorLayer | null = null
  /** 事件处理 dispose（清理定时器） */
  let disposeEvents: (() => void) | null = null

  // WorkerPool（在 onMounted 中初始化）
  let pool: WorkerPool | null = null

  // ===== 当前种子/版本/维度（用于 tile loader） =====
  let currentSeed = ''
  let currentMc = 34
  let currentDim: Dimension = 0
  let currentLargeBiomes = false

  // ===== 工厂：tile 加载器 =====
  const loadBiomeTile = createTileLoader(() => ({
    seed: currentSeed,
    mcVersion: currentMc,
    dimension: currentDim,
    largeBiomes: currentLargeBiomes,
    yCoord: yCoord.value,
    doContour: doContour.value,
    ymaxLimit: ymaxLimit.value,
    pool,
  }))

  // ===== 工厂：结构管理器 =====
  const {
    structureListForVersion,
    refreshStructures,
    refreshSpecials,
    renderStructures,
    toggleStructureType,
    isStructureSelected,
  } = createStructureManager({
    structures,
    selectedStructureTypes,
    showNonViable,
    mcVersion,
    dimension,
    getMap: () => map, getPool: () => pool,
    getStructSource: () => structSource, getSpawnSource: () => spawnSource,
    getStrongholdSource: () => strongholdSource,
    getCurrentSeed: () => currentSeed, getCurrentMc: () => currentMc,
    getCurrentDim: () => currentDim, getCurrentLargeBiomes: () => currentLargeBiomes,
  })

  // ===== 结构图层可见性控制 =====
  function updateStructLayerVisibility() {
    if (!map || !structLayerRef) return
    structLayerRef.setVisible((map.getView().getZoom() ?? 0) >= STRUCT_MIN_ZOOM)
  }

  // ===== 创建 OL Map =====
  function initMap() {
    if (!mapContainer.value) return
    const init = initSeedMap({
      target: mapContainer.value,
      loader: ((z: number, x: number, y: number) => loadBiomeTile(z, x, y)) as unknown as DataTileLoader,
      popupContainer: popupContainer.value,
      highlight,
    })
    ;({ map, biomeSource, biomeLayer, structSource, structLayer: structLayerRef,
       spawnSource, spawnLayer, strongholdSource, strongholdLayer, clickMarkerSource,
       popupOverlay } = init)

    disposeEvents = createMapEvents({
      hoverStruct, hoverMarker, mouseBlock, lastClickBlock, popupData, mouseBiomeName,
      showSpawn, showStronghold, dimension, yCoord, highlight,
      getMap: () => map,
      getStructSource: () => structSource,
      getSpawnSource: () => spawnSource,
      getStrongholdSource: () => strongholdSource,
      getClickMarkerSource: () => clickMarkerSource,
      getPopupOverlay: () => popupOverlay,
      getCurrentSeed: () => currentSeed,
      getCurrentMc: () => currentMc,
      getCurrentDim: () => currentDim,
      getCurrentLargeBiomes: () => currentLargeBiomes,
      getPool: () => pool,
      updateStructLayerVisibility,
      refreshStructures,
      closePopup,
    })
  }

  // ===== 加载种子 =====
  async function loadSeed(seedOverride?: string) {
    const seed = seedOverride ?? seedInput.value.trim()
    if (!seed) { toastError('请输入种子'); return }
    if (!pool) { toastError('WorkerPool 未初始化'); return }
    currentSeed = seed
    currentMc = mcVersion.value
    currentDim = dimension.value
    currentLargeBiomes = largeBiomes.value
    map?.getView().setCenter([0, 0])
    map?.getView().setZoom(6)
    biomeSource?.refresh()
    biomeLayer?.setVisible(true)
    updateStructLayerVisibility()
    structSource?.clear()
    spawnSource?.clear()
    strongholdSource?.clear()
    clickMarkerSource?.clear()
    structures.value = []
    closePopup()
    hoverStruct.value = null
    lastClickBlock.value = null
    mouseBiomeName.value = ''
    loading.value = true
    try {
      await pool.prepareSeed(currentSeed, currentMc, currentDim, currentLargeBiomes)
    } catch {
      // prepareSeed 失败不阻塞（tile loader 会再触发）
    }
    refreshStructures()
    refreshSpecials()
    setTimeout(() => { loading.value = false }, 500)
    toastSuccess('已加载种子: ' + currentSeed.slice(0, 24))
  }

  // ===== 前往用户坐标 =====
  function goToUserCoord() {
    const x = parseInt(userX.value), z = parseInt(userZ.value)
    if (isNaN(x) || isNaN(z)) { toastError('请输入有效坐标'); return }
    if (!map) return
    map.getView().animate({ center: [x, z], zoom: 8, duration: 500 })
    toastSuccess(`已前往 (${x}, ${z})`)
  }

  // ===== 缩放按钮 =====
  function zoomIn() { map?.getView().setZoom(Math.min(MAX_ZOOM, (map?.getView().getZoom() ?? 0) + 1)) }
  function zoomOut() { map?.getView().setZoom(Math.max(MIN_ZOOM, (map?.getView().getZoom() ?? 0) - 1)) }
  function resetView() { map?.getView().animate({ center: [0, 0], zoom: 6, duration: 300 }) }

  // ===== Popup 浮窗交互 =====
  function closePopup(): void { popupData.value = null; popupOverlay?.setPosition(undefined) }

  function goToStruct(x: number, z: number): void { if (!map) return; map.getView().animate({ center: [x, z], zoom: 8, duration: 500 }) }

  async function copyCoord(x: number, z: number): Promise<void> {
    const text = formatCoord(x, z), ok = await copyToClipboard(text)
    ok ? toastSuccess('已复制: ' + text) : toastError('复制失败')
  }

  // ===== 版本/维度/大型群系变化 =====
  watch([mcVersion, dimension, largeBiomes], () => {
    const available = new Set(
      getStructuresForVersion(mcVersion.value, dimension.value)
        .filter((s) => s.queryMode !== 'stronghold')
        .map((s) => s.name),
    )
    const invalid: string[] = []
    for (const name of selectedStructureTypes.value) {
      if (!available.has(name)) invalid.push(name)
    }
    if (invalid.length > 0) {
      const next = new Set(selectedStructureTypes.value)
      for (const name of invalid) next.delete(name)
      selectedStructureTypes.value = next
    }
    if (!currentSeed) return
    currentMc = mcVersion.value
    currentDim = dimension.value
    currentLargeBiomes = largeBiomes.value
    biomeSource?.refresh()
    structSource?.clear()
    refreshStructures()
    refreshSpecials()
  })

  watch(showSpawn, (v) => { if (spawnLayer) spawnLayer.setVisible(v) })
  watch(showStronghold, (v) => { if (strongholdLayer) strongholdLayer.setVisible(v) })
  watch(showNonViable, () => renderStructures(structures.value))
  watch([yCoord, doContour, ymaxLimit], () => { if (!currentSeed) return; biomeSource?.refresh() })

  // ===== 生命周期 =====
  onMounted(async () => {
    await nextTick()
    initMap()
    try {
      pool = new WorkerPool()
      await pool.init(resUrl('cubiomes.js'), resUrl('cubiomes.wasm'))
      await loadSeed(DEFAULT_SEED)
    } catch (e) {
      toastError('WorkerPool 初始化失败: ' + (e instanceof Error ? e.message : String(e)))
    }
  })
  onBeforeUnmount(() => {
    disposeEvents?.()
    disposeEvents = null
    pool?.dispose()
    pool = null
    if (map) { map.setTarget(undefined); map = null }
  })

  return {
    seedInput, mcVersion, dimension, largeBiomes, userX, userZ,
    versionOptions, dimensionOptions,
    loading, structures, hoverStruct, hoverMarker, mouseBlock, lastClickBlock,
    popupData, mouseBiomeName,
    mapContainer, popupContainer,
    showSpawn, showStronghold, showCoordPanel, showNonViable,
    yCoord, doContour, ymaxLimit,
    selectedStructureTypes, structureListForVersion,
    loadSeed, goToUserCoord, zoomIn, zoomOut, resetView,
    copyCoord, goToStruct, closePopup,
    getStructIcon, getStructIconUrl, toggleStructureType, isStructureSelected,
  }
}
