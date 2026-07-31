/**
 * 种子地图组合式逻辑（OpenLayers + WASM Worker 版）
 *
 * 架构（参考 minecraftsearch.com 逆向分析 docs/Map/map.md）：
 * - 投影：自定义 'mc' 投影，1 单位 = 1 方块，extent ±3e7
 * - 群系图层：DataTile source，loader 调 WorkerPool.generateTile 获取 ImageBitmap
 *   OL 自动按 (z,x,y) 缓存 tile，已加载的区块不再重新请求
 * - 结构图层：VectorLayer + Feature，moveend 时按可视范围调 WorkerPool.findStructures
 * - 出生点/多座要塞：VectorLayer，loadSeed 时调 WorkerPool.getSpecials
 * - 交互：OL 内置拖拽/缩放/惯性；pointermove/singleclick 事件做几何 hit detection
 *
 * 按职责拆分到 `./useSeedMap/` 子文件：
 * - `config.ts`：Zoom/extent 常量 + SEEDMAP_MC_VERSIONS + mapMcVersionToCubiomes
 * - `tileLoader.ts`：群系 tile 加载（createTileLoader 工厂）
 * - `structureManager.ts`：结构加载/渲染/筛选（createStructureManager 工厂）
 *
 * initMap + 事件处理 + 生命周期保留在主文件（与 composable 闭包紧密耦合）。
 */
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { toastError, toastSuccess } from '@/utils/toast'
import {
  getStructStyle, getMarkerStyle, getClickMarkerStyle, getStructIcon, getStructIconUrl,
} from '@/utils/seedmap/constants'
import { WorkerPool } from '@/utils/seedmap/workerPool'
import { resUrl } from '@/utils/wasm-loader'
import { formatCoord, copyToClipboard } from '@/utils/seedmap/format'
import { getBiomeName } from '@/utils/seedmap/biomeNames'
import type { Dimension, WorkerStructure } from '@/utils/seedmap/types'

import OlMap from 'ol/Map'
import View from 'ol/View'
import Projection from 'ol/proj/Projection'
import DataTile, { type Loader as DataTileLoader } from 'ol/source/DataTile'
import TileLayer from 'ol/layer/Tile'
import VectorLayer from 'ol/layer/Vector'
import VectorSource from 'ol/source/Vector'
import Feature from 'ol/Feature'
import Point from 'ol/geom/Point'
import TileGrid from 'ol/tilegrid/TileGrid'
import Overlay from 'ol/Overlay'
import { defaults as defaultInteractions } from 'ol/interaction'
import MouseWheelZoom from 'ol/interaction/MouseWheelZoom'

// 从子模块导入配置与工厂
import {
  MIN_ZOOM, MAX_ZOOM, TILE_SIZE, RESOLUTIONS, EXTENT, EXTENT_HALF,
  STRUCT_MIN_ZOOM, SEEDMAP_MC_VERSIONS,
} from './useSeedMap/config'
// re-export mapMcVersionToCubiomes 保持调用方路径兼容
export { mapMcVersionToCubiomes } from './useSeedMap/config'
import { createTileLoader } from './useSeedMap/tileLoader'
import { createStructureManager } from './useSeedMap/structureManager'

export function useSeedMap() {
  // ===== 控件状态 =====
  const seedInput = ref<string>('')
  const mcVersion = ref<number>(28)  // 默认最新版（26.2 = cubiomes MC_26_2 = MC_NEWEST = 28）
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
  const dimensionOptions = [
    { label: '主世界', value: 0 },
    { label: '下界', value: -1 },
    { label: '末地', value: 1 },
  ]

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

  // OL 实例（非响应式，用 let）
  const mapContainer = ref<HTMLDivElement | null>(null)
  const popupContainer = ref<HTMLDivElement | null>(null)
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
  let hoverFeat: Feature | null = null
  let clickFeat: Feature | null = null
  let biomeDebounce: number | null = null
  let structLayerRef: VectorLayer | null = null

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
    getMap: () => map,
    getPool: () => pool,
    getStructSource: () => structSource,
    getSpawnSource: () => spawnSource,
    getStrongholdSource: () => strongholdSource,
    getCurrentSeed: () => currentSeed,
    getCurrentMc: () => currentMc,
    getCurrentDim: () => currentDim,
    getCurrentLargeBiomes: () => currentLargeBiomes,
  })

  // ===== 结构图层可见性控制 =====
  function updateStructLayerVisibility() {
    if (!map || !structLayerRef) return
    const zoom = map.getView().getZoom() ?? 0
    const visible = zoom >= STRUCT_MIN_ZOOM
    structLayerRef.setVisible(visible)
  }

  // ===== 创建 OL Map =====
  function initMap() {
    if (!mapContainer.value) return

    const projection = new Projection({
      code: 'mc',
      units: 'm',
      extent: EXTENT,
    })

    biomeSource = new DataTile({
      tileGrid: new TileGrid({
        extent: EXTENT,
        origin: [-EXTENT_HALF, EXTENT_HALF],
        resolutions: RESOLUTIONS,
        tileSize: TILE_SIZE,
      }),
      projection,
      loader: ((z: number, x: number, y: number) => loadBiomeTile(z, x, y)) as unknown as DataTileLoader,
      wrapX: false,
      transition: 0,
    })
    biomeLayer = new TileLayer({ source: biomeSource, visible: false, cacheSize: 4096, preload: 1 })

    structSource = new VectorSource()
    const structLayer = new VectorLayer({
      source: structSource,
      style: (feature) => {
        const stype = feature.get('stype') as string
        const highlighted = feature === hoverFeat || feature === clickFeat
        return getStructStyle(stype, highlighted)
      },
    })
    structLayerRef = structLayer

    spawnSource = new VectorSource()
    spawnLayer = new VectorLayer({ source: spawnSource, style: getMarkerStyle('#4CAF50') })

    strongholdSource = new VectorSource()
    strongholdLayer = new VectorLayer({ source: strongholdSource, style: getMarkerStyle('#E91E63') })

    clickMarkerSource = new VectorSource()
    const clickLayer = new VectorLayer({
      source: clickMarkerSource,
      style: getClickMarkerStyle(),
    })

    if (popupContainer.value) {
      popupOverlay = new Overlay({
        element: popupContainer.value,
        autoPan: { animation: { duration: 250 }, margin: 16 },
        offset: [0, -16],
        positioning: 'bottom-center',
      })
    }

    map = new OlMap({
      target: mapContainer.value,
      layers: [biomeLayer, structLayer, spawnLayer, strongholdLayer, clickLayer],
      ...(popupOverlay ? { overlays: [popupOverlay] } : {}),
      view: new View({
        projection,
        center: [0, 0],
        zoom: 6,
        minZoom: MIN_ZOOM,
        maxZoom: MAX_ZOOM,
        extent: EXTENT,
        constrainOnlyCenter: true,
        constrainResolution: false,
        resolutions: RESOLUTIONS,
      }),
      controls: [],
      interactions: defaultInteractions({
        mouseWheelZoom: false,
      }).extend([
        new MouseWheelZoom({ useAnchor: true, maxDelta: 1, duration: 250 }),
      ]),
    })

    // 几何 hit detection：遍历 feature 计算像素距离，避免 getImageData 警告
    const HIT_TOLERANCE_PX = 6
    type HitType = 'struct' | 'spawn' | 'stronghold'
    interface HitResult {
      feature: Feature
      type: HitType
      label: string
      x: number
      z: number
    }
    function findFeatureAtPixel(pixel: number[]): HitResult | null {
      if (!map) return null
      const resolution = map.getView().getResolution() ?? 1
      const tolCoord = HIT_TOLERANCE_PX * resolution
      const [cx, cz] = map.getCoordinateFromPixel(pixel)
      let best: HitResult | null = null
      let bestDist = Infinity
      const checkSource = (source: VectorSource | null, type: HitType, enabled: boolean) => {
        if (!source || !enabled) return
        source.forEachFeature((feat) => {
          const geom = feat.getGeometry()
          if (!geom || geom.getType() !== 'Point') return
          const [fx, fz] = (geom as Point).getCoordinates()
          const dx = fx - cx
          const dz = fz - cz
          const dist = dx * dx + dz * dz
          if (dist <= tolCoord * tolCoord && dist < bestDist) {
            bestDist = dist
            if (type === 'struct') {
              const struct = feat.get('data') as WorkerStructure
              best = { feature: feat, type, label: getStructIcon(struct.stype).label, x: struct.x, z: struct.z }
            } else {
              best = { feature: feat, type, label: type === 'spawn' ? '出生点' : '要塞', x: Math.round(fx), z: Math.round(fz) }
            }
          }
        })
      }
      checkSource(structSource, 'struct', true)
      checkSource(spawnSource, 'spawn', showSpawn.value)
      checkSource(strongholdSource, 'stronghold', showStronghold.value && dimension.value === 0)
      return best
    }

    map.on('singleclick', (e) => {
      const hit = findFeatureAtPixel(e.pixel)
      if (hit) {
        if (hit.type === 'struct') {
          clickFeat = hit.feature
          const struct = hit.feature.get('data') as WorkerStructure
          popupData.value = { struct, coord: [struct.x, struct.z] }
          popupOverlay?.setPosition([struct.x, struct.z])
          structSource!.changed()
        } else {
          clickFeat = null
          closePopup()
          lastClickBlock.value = { x: hit.x, z: hit.z }
          clickMarkerSource!.clear()
          clickMarkerSource!.addFeature(new Feature({ geometry: new Point([hit.x, hit.z]) }))
        }
      } else {
        clickFeat = null
        closePopup()
        const [cx, cz] = e.coordinate
        lastClickBlock.value = { x: Math.round(cx), z: Math.round(cz) }
        clickMarkerSource!.clear()
        clickMarkerSource!.addFeature(new Feature({ geometry: new Point([cx, cz]) }))
      }
    })

    let hoverThrottle: number | null = null
    let lastPixel: number[] | null = null
    map.on('pointermove', (e) => {
      if (e.dragging) return
      const [cx, cz] = e.coordinate
      const blockX = Math.round(cx)
      const blockZ = Math.round(cz)
      mouseBlock.value = { x: blockX, z: blockZ }
      lastPixel = e.pixel
      if (hoverThrottle) return
      hoverThrottle = window.setTimeout(() => {
        hoverThrottle = null
        if (!lastPixel) return
        const hit = findFeatureAtPixel(lastPixel)
        if (hit) {
          if (hit.type === 'struct') {
            if (hit.feature !== hoverFeat) {
              hoverFeat = hit.feature
              hoverStruct.value = hit.feature.get('data') as WorkerStructure
              hoverMarker.value = null
              structSource!.changed()
            }
          } else {
            if (hoverFeat !== null) {
              hoverFeat = null
              hoverStruct.value = null
              structSource!.changed()
            }
            hoverMarker.value = { label: hit.label, x: hit.x, z: hit.z }
          }
        } else {
          if (hoverFeat !== null) {
            hoverFeat = null
            hoverStruct.value = null
            structSource!.changed()
          }
          hoverMarker.value = null
        }
      }, 50)
      scheduleBiomeQuery(blockX, blockZ)
    })

    let moveendTimer: number | null = null
    map.on('moveend', () => {
      updateStructLayerVisibility()
      if (!currentSeed) return
      if (moveendTimer) clearTimeout(moveendTimer)
      moveendTimer = window.setTimeout(() => {
        moveendTimer = null
        refreshStructures()
      }, 300)
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
    if (map) {
      map.getView().setCenter([0, 0])
      map.getView().setZoom(6)
    }
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
    const x = parseInt(userX.value)
    const z = parseInt(userZ.value)
    if (isNaN(x) || isNaN(z)) { toastError('请输入有效坐标'); return }
    if (!map) return
    map.getView().animate({ center: [x, z], zoom: 8, duration: 500 })
    toastSuccess(`已前往 (${x}, ${z})`)
  }

  // ===== 缩放按钮 =====
  function zoomIn() { map?.getView().setZoom(Math.min(MAX_ZOOM, (map?.getView().getZoom() ?? 0) + 1)) }
  function zoomOut() { map?.getView().setZoom(Math.max(MIN_ZOOM, (map?.getView().getZoom() ?? 0) - 1)) }
  function resetView() {
    map?.getView().animate({ center: [0, 0], zoom: 6, duration: 300 })
  }

  // ===== Popup 浮窗交互 =====
  function closePopup(): void {
    popupData.value = null
    popupOverlay?.setPosition(undefined)
  }

  function goToStruct(x: number, z: number): void {
    if (!map) return
    map.getView().animate({ center: [x, z], zoom: 8, duration: 500 })
  }

  async function copyCoord(x: number, z: number): Promise<void> {
    const text = formatCoord(x, z)
    const ok = await copyToClipboard(text)
    if (ok) toastSuccess('已复制: ' + text)
    else toastError('复制失败')
  }

  /** 鼠标悬停查询群系名（debounce 300ms） */
  function scheduleBiomeQuery(blockX: number, blockZ: number): void {
    if (biomeDebounce) clearTimeout(biomeDebounce)
    biomeDebounce = window.setTimeout(async () => {
      biomeDebounce = null
      if (!currentSeed || !pool) {
        mouseBiomeName.value = ''
        return
      }
      try {
        const biomeId = await pool.getBiomeAtPoint({
          seed: currentSeed,
          mcVersion: currentMc,
          dimension: currentDim,
          largeBiomes: currentLargeBiomes,
          scale: 4,
          x: Math.round(blockX / 4),
          y: Math.round(yCoord.value / 4),
          z: Math.round(blockZ / 4),
        })
        mouseBiomeName.value = biomeId >= 0 ? getBiomeName(biomeId) : ''
      } catch {
        mouseBiomeName.value = ''
      }
    }, 300)
  }

  // ===== 版本/维度/大型群系变化 =====
  watch([mcVersion, dimension, largeBiomes], () => {
    const available = new Set(
      require('@/utils/seedmap/structures').getStructuresForVersion(mcVersion.value, dimension.value)
        .filter((s: { queryMode: string }) => s.queryMode !== 'stronghold')
        .map((s: { name: string }) => s.name),
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
  watch([yCoord, doContour, ymaxLimit], () => {
    if (!currentSeed) return
    biomeSource?.refresh()
  })

  // ===== 生命周期 =====
  onMounted(async () => {
    await nextTick()
    initMap()
    try {
      const wasmJsUrl = resUrl('cubiomes.js')
      const wasmUrl = resUrl('cubiomes.wasm')
      pool = new WorkerPool()
      await pool.init(wasmJsUrl, wasmUrl)
      await loadSeed(DEFAULT_SEED)
    } catch (e) {
      toastError('WorkerPool 初始化失败: ' + (e instanceof Error ? e.message : String(e)))
    }
  })
  onBeforeUnmount(() => {
    if (biomeDebounce) {
      clearTimeout(biomeDebounce)
      biomeDebounce = null
    }
    pool?.dispose()
    pool = null
    if (map) {
      map.setTarget(undefined)
      map = null
    }
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
