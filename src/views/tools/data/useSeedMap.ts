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
 * zoom 体系（与原站对齐）：
 * - OL zoom 0~12（共 13 级）
 * - RESOLUTIONS = [256,128,64,32,16,8,4,2,1,0.5,0.25,0.125,0.0625]（方块/像素）
 * - tile 64×64 像素，每 tile 覆盖方块 = 64 × resolution
 * - cubiomes scale ∈ {1,4,16,64,256}，每级 zoom 取 ≤ bpp 的最大 scale 生成 biome
 *   （非 power-of-4 的 bpp 通过 sx/sz > TILE_SIZE 由 worker 降采样适配）
 */
import { ref, onMounted, onBeforeUnmount, watch, computed, nextTick } from 'vue'
import { toastError, toastSuccess } from '@/utils/toast'
import {
  getStructStyle, getMarkerStyle, getClickMarkerStyle, getStructIcon, getStructIconUrl,
} from '@/utils/seedmap/constants'
import { WorkerPool } from '@/utils/seedmap/workerPool'
import { resUrl } from '@/utils/wasm-loader'
import { getStructuresForVersion } from '@/utils/seedmap/structures'
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

// ===== Zoom level 配置（与原站对齐） =====
const MIN_ZOOM = 0
const MAX_ZOOM = 12
const TILE_SIZE = 64  // 原站用 64，不是 256
// 13 级 resolution：从 256 bpp（最远）到 0.0625 bpp（最近，16 像素一个方块）
// 比原站多 2 级（0.125, 0.0625），让用户能放大到看清方块细节
const RESOLUTIONS = [256, 128, 64, 32, 16, 8, 4, 2, 1, 0.5, 0.25, 0.125, 0.0625]

// 投影 extent：约 ±3e7 方块
// EXTENT_HALF 必须是最大 blocksPerTile（64×256=16384=2^14）的整数倍，
// 确保所有 zoom 级别的 tile 边界与 extent 完全对齐，避免相邻 tile 内容不连续
const EXTENT_HALF = 29_999_104  // 16384 × 1831 = 2^14 × 1831
const EXTENT = [-EXTENT_HALF, -EXTENT_HALF, EXTENT_HALF, EXTENT_HALF]

/**
 * MC 版本列表（与 cubiomes MC_* 枚举值映射；见 cubiomes/biomes.h:5-46）
 *
 * 版本支持说明：
 * - 使用 fork 仓库 https://github.com/MoTeam-cn/cubiomes，原生支持 MC_26_2 (=34)
 * - 枚举值（biomes.h）：
 *   MC_1_21_WD=28 (1.21.4), MC_1_21_5=29, MC_1_21_6=30, MC_1_21_9=31,
 *   MC_1_21_11=32 (=MC_1_21), MC_26_1=33, MC_26_2=34 (=MC_NEWEST)
 * - 1.21.9/1.21.11 共用 31（cubiomes 未单独定义 1.21.11 枚举，1.21.9=31 已涵盖）
 */
const SEEDMAP_MC_VERSIONS = [
  // Latest（fork cubiomes 原生支持 MC_26_2）
  { label: '26.2', value: 34 },
  { label: '26.1', value: 33 },
  { label: '1.21.9', value: 31 },
  { label: '1.21.6', value: 30 },
  { label: '1.21.5', value: 29 },
  { label: '1.21.4', value: 28 },
  { label: '1.21.3', value: 27 },
  { label: '1.21.1', value: 26 },
  { label: '1.20', value: 25 },
  { label: '1.19.4', value: 24 },
  { label: '1.19.2', value: 23 },
  { label: '1.18', value: 22 },
  // Old
  { label: '1.17', value: 21 }, { label: '1.16', value: 20 },
  { label: '1.15', value: 18 }, { label: '1.14', value: 17 },
  { label: '1.13', value: 16 }, { label: '1.12', value: 15 },
  { label: '1.11', value: 14 }, { label: '1.10', value: 13 },
  { label: '1.9', value: 12 }, { label: '1.8', value: 11 },
  { label: '1.7', value: 10 },
] as const

export function useSeedMap() {
  // ===== 控件状态 =====
  const seedInput = ref<string>('')
  const mcVersion = ref<number>(34)  // 默认最新版（26.2 = cubiomes MC_26_2 = MC_NEWEST）
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
  const mouseBlock = ref<{ x: number; z: number } | null>(null)
  const lastClickBlock = ref<{ x: number; z: number } | null>(null)
  /** popup 浮窗数据（含 OL 坐标用于 Overlay 定位） */
  const popupData = ref<{ struct: WorkerStructure; coord: [number, number] } | null>(null)
  /** 鼠标悬停的群系名（异步查询，debounce 300ms） */
  const mouseBiomeName = ref<string>('')
  const showSpawn = ref<boolean>(true)
  const showStronghold = ref<boolean>(true)
  const selectedStructureTypes = ref<Set<string>>(new Set(['Village']))

  // OL 实例（非响应式，用 let）
  const mapContainer = ref<HTMLDivElement | null>(null)
  /** OL Overlay 的容器 ref（必须在 initMap 前已渲染） */
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
  // hover/click 高亮的 feature（由 pointermove/singleclick 事件设置）
  let hoverFeat: Feature | null = null
  let clickFeat: Feature | null = null
  /** biome 名称查询 debounce 句柄 */
  let biomeDebounce: number | null = null

  // WorkerPool（在 onMounted 中初始化）
  let pool: WorkerPool | null = null

  // ===== 当前种子/版本/维度（用于 tile loader） =====
  let currentSeed = ''
  let currentMc = 34
  let currentDim: Dimension = 0
  let currentLargeBiomes = false

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
        // origin 在左上角（北方=大 Z，西方=小 X）= OL 标准 XYZ 方案
        // tile y=0 对应最北端，y 递增向南
        origin: [-EXTENT_HALF, EXTENT_HALF],
        resolutions: RESOLUTIONS,
        tileSize: TILE_SIZE,
      }),
      projection,
      loader: ((z: number, x: number, y: number) => loadBiomeTile(z, x, y)) as unknown as DataTileLoader,
      wrapX: false,
      transition: 0,
    })
    // preload:1 预加载相邻 zoom 级别的 tile，减少拖拽/缩放时的边缘空白
    // cacheSize 增大到 4096 避免大范围浏览时 tile 被过早清除导致重新加载
    // transition:0 禁用淡入动画，tile 生成后立即显示（WASM 生成已有延迟，无需额外过渡）
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

    spawnSource = new VectorSource()
    spawnLayer = new VectorLayer({ source: spawnSource, style: getMarkerStyle('#4CAF50') })

    strongholdSource = new VectorSource()
    strongholdLayer = new VectorLayer({ source: strongholdSource, style: getMarkerStyle('#E91E63') })

    clickMarkerSource = new VectorSource()
    const clickLayer = new VectorLayer({
      source: clickMarkerSource,
      style: getClickMarkerStyle(),
    })

    // OL Overlay：渲染结构点击 popup（element 来自 Vue ref popupContainer）
    // 必须在 onMounted 中 await nextTick() 后调用 initMap，确保 popupContainer 已挂载
    // OL v10 API：autoPan 直接传 PanIntoViewOptions 对象（含 animation + margin）
    if (popupContainer.value) {
      popupOverlay = new Overlay({
        element: popupContainer.value,
        autoPan: { animation: { duration: 250 }, margin: 16 },
        offset: [0, -16],  // 上偏移 16px，避免遮盖结构图标
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
        // 默认 zoom 6 → resolution 4 bpp（scale=4，平衡细节与性能）
        zoom: 6,
        minZoom: MIN_ZOOM,
        maxZoom: MAX_ZOOM,
        extent: EXTENT,
        constrainOnlyCenter: true,
        // 不强制离散 zoom，允许滚轮停在非整数 zoom，tile 会用最近级别拉伸
        // 配合 image-rendering: pixelated 保持像素清晰
        constrainResolution: false,
        resolutions: RESOLUTIONS,
      }),
      controls: [],
      interactions: defaultInteractions({
        mouseWheelZoom: false,  // 用自定义的 MouseWheelZoom 替代默认的
      }).extend([
        // MouseWheelZoom 配置：
        // - useAnchor: 围绕鼠标位置缩放
        // - maxDelta: 限制单次滚轮缩放幅度，避免一次跳多级 zoom
        // - duration: 缩放动画时长，平滑过渡
        new MouseWheelZoom({ useAnchor: true, maxDelta: 1, duration: 250 }),
      ]),
    })

    // 几何 hit detection：直接遍历 structSource 的 feature，计算像素距离
    // 避免 OL Select 交互的 forEachFeatureAtPixel → getImageData 触发 willReadFrequently 警告
    // HIT_TOLERANCE_PX 为像素容差，与 OL Select 默认 hitTolerance 一致
    const HIT_TOLERANCE_PX = 6
    function findStructAtPixel(pixel: number[]): Feature | null {
      if (!map || !structSource) return null
      const resolution = map.getView().getResolution() ?? 1
      // 像素容差转坐标容差（resolution = 方块/像素）
      const tolCoord = HIT_TOLERANCE_PX * resolution
      let best: Feature | null = null
      let bestDist = Infinity
      structSource.forEachFeature((feat) => {
        const geom = feat.getGeometry()
        if (!geom || geom.getType() !== 'Point') return
        const [fx, fz] = (geom as Point).getCoordinates()
        const [cx, cz] = map!.getCoordinateFromPixel(pixel)
        const dx = fx - cx
        const dz = fz - cz
        const dist = dx * dx + dz * dz
        if (dist <= tolCoord * tolCoord && dist < bestDist) {
          bestDist = dist
          best = feat
        }
      })
      return best
    }

    map.on('singleclick', (e) => {
      const feat = findStructAtPixel(e.pixel)
      if (feat) {
        clickFeat = feat
        const struct = feat.get('data') as WorkerStructure
        popupData.value = { struct, coord: [struct.x, struct.z] }
        popupOverlay?.setPosition([struct.x, struct.z])
        structSource!.changed()
      } else {
        // 点击空白：标记坐标 + 关闭 popup
        clickFeat = null
        closePopup()
        const [cx, cz] = e.coordinate
        lastClickBlock.value = { x: Math.round(cx), z: Math.round(cz) }
        clickMarkerSource!.clear()
        clickMarkerSource!.addFeature(new Feature({ geometry: new Point([cx, cz]) }))
      }
    })
    // pointermove 节流（避免高频遍历 feature）
    let hoverThrottle: number | null = null
    map.on('pointermove', (e) => {
      if (e.dragging) return
      const [cx, cz] = e.coordinate
      const blockX = Math.round(cx)
      const blockZ = Math.round(cz)
      mouseBlock.value = { x: blockX, z: blockZ }
      if (hoverThrottle) return
      hoverThrottle = window.setTimeout(() => {
        hoverThrottle = null
        const feat = findStructAtPixel(e.pixel)
        if (feat !== hoverFeat) {
          hoverFeat = feat
          hoverStruct.value = feat ? (feat.get('data') as WorkerStructure) : null
          structSource!.changed()
        }
      }, 50)
      // biome 名称查询：debounce 300ms
      scheduleBiomeQuery(blockX, blockZ)
    })

    let moveendTimer: number | null = null
    map.on('moveend', () => {
      if (!currentSeed) return
      if (moveendTimer) clearTimeout(moveendTimer)
      moveendTimer = window.setTimeout(() => {
        moveendTimer = null
        refreshStructures()
      }, 300)
    })
  }

  // ===== 加载群系 tile =====
  // 坐标系约定（与 generatorWorker.ts 对齐）：
  // - OL TileGrid 用 top-left origin = [-EXTENT_HALF, +EXTENT_HALF]
  // - tile y=0 在屏幕顶部 = 投影 max Y = MC max Z（本项目约定 +Z=北方）
  // - startBlockZ 取 tile 的 min Z（南方边缘），由 worker 翻转 Z 轴后渲染
  // - blockX/blockZ 始终是 **方块坐标**；worker 内部负责 block → scale 坐标转换
  //   （cubiomes Range.x/z 期望 scale 坐标，见 cubiomes/biomenoise.h Range 注释）
  async function loadBiomeTile(z: number, x: number, y: number): Promise<ImageBitmap> {
    const emptyBitmap = async () => {
      const c = document.createElement('canvas')
      c.width = c.height = TILE_SIZE
      return createImageBitmap(c)
    }
    if (!currentSeed || !pool) return emptyBitmap()
    // z 直接对应 RESOLUTIONS 索引（OL zoom 0~12）
    const res = RESOLUTIONS[z]
    if (!res) return emptyBitmap()
    const bpp = res // 方块/像素
    // cubiomes 只支持 scale ∈ {1, 4, 16, 64, 256}（见 cubiomes/biomenoise.h:15）
    // 选择不超过 bpp 的最大支持值，确保采样密度合适
    const SUPPORTED_SCALES = [1, 4, 16, 64, 256]
    const scale: number = SUPPORTED_SCALES.filter(s => s <= bpp).pop() ?? 1
    // sx/sz = tile 覆盖的方块数 / scale = scale 坐标系下的采样数
    // - 当 bpp == scale（如 bpp=4, scale=4）：sx = TILE_SIZE，1 sample/pixel
    // - 当 bpp > scale（如 bpp=8, scale=4）：sx = 2*TILE_SIZE，2 samples/pixel（worker 降采样）
    // - 当 bpp < scale（如 bpp=0.5, scale=1）：sx = TILE_SIZE/2，0.5 samples/pixel（worker 升采样）
    const sx = Math.min(2048, Math.max(1, Math.round((TILE_SIZE * bpp) / scale)))
    const sz = Math.min(2048, Math.max(1, Math.round((TILE_SIZE * bpp) / scale)))
    const blocksPerTile = TILE_SIZE * bpp
    // blockX = -EXTENT_HALF + tileX × blocksPerTile（西→东，tile 左上角方块 X）
    // blockZ = EXTENT_HALF - (tileY+1) × blocksPerTile（北→南，tile 的 min Z = 南边缘）
    // 注意：EXTENT_HALF 和 blocksPerTile 都是 scale 的整数倍，所以 startBlockX/Z 必然
    // 能被 scale 整除，worker 内 blockX/scale、blockZ/scale 为整数，tile 边界与
    // cubiomes scale 网格完全对齐，相邻 tile 内容连续。
    const startBlockX = Math.round(-EXTENT_HALF + x * blocksPerTile)
    const startBlockZ = Math.round(EXTENT_HALF - (y + 1) * blocksPerTile)
    const tileParams = {
      seed: currentSeed,
      mcVersion: currentMc,
      dimension: currentDim,
      largeBiomes: currentLargeBiomes,
      blockX: startBlockX,
      blockZ: startBlockZ,
      sx, sz, scale,
      y: yCoord.value,
      doContour: doContour.value,
      ymax: ymaxLimit.value > 0 ? ymaxLimit.value : Infinity,
    }
    // 重试机制：Worker 可能因 WASM 初始化未完成或内存增长偶发失败，
    // 重试 2 次（共 3 次尝试，间隔 200ms），避免永久空缺需缩放才触发重载
    const MAX_RETRIES = 2
    for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
      try {
        return await pool.generateTile(tileParams)
      } catch (e) {
        if (attempt < MAX_RETRIES) {
          await new Promise(r => setTimeout(r, 200))
          continue
        }
        console.warn('tile load failed after retries', z, x, y, e)
        return emptyBitmap()
      }
    }
    return emptyBitmap()
  }

  // ===== 加载结构（按可视范围） =====
  let structRequestId = 0
  async function refreshStructures() {
    if (!currentSeed || !map || !pool) return
    const myId = ++structRequestId
    try {
      const view = map.getView()
      const extent = view.calculateExtent(map.getSize())
      const [minX, minZ, maxX, maxZ] = extent
      const margin = 256
      const items = await pool.findStructures({
        seed: currentSeed,
        mcVersion: currentMc,
        dimension: currentDim,
        largeBiomes: currentLargeBiomes,
        minX: Math.round(minX) - margin,
        minZ: Math.round(minZ) - margin,
        maxX: Math.round(maxX) + margin,
        maxZ: Math.round(maxZ) + margin,
      })
      if (myId !== structRequestId) return
      structures.value = items
      renderStructures(items)
    } catch (e) {
      toastError('结构加载失败: ' + (e instanceof Error ? e.message : String(e)))
    }
  }

  // ===== 加载特殊点（出生点/多座要塞） =====
  // cubiomes nextStronghold 迭代最多返回 128 座要塞，OL 自动渲染所有 Feature。
  async function refreshSpecials() {
    if (!currentSeed || !spawnSource || !strongholdSource || !pool) return
    try {
      const res = await pool.getSpecials({
        seed: currentSeed,
        mcVersion: currentMc,
        largeBiomes: currentLargeBiomes,
      })
      spawnSource.clear()
      if (res.spawn) {
        const feat = new Feature({ geometry: new Point([res.spawn.x, res.spawn.z]) })
        spawnSource.addFeature(feat)
      }
      strongholdSource.clear()
      // 遍历 strongholds 数组添加多个 Feature（要塞数量上限 128）
      for (const sh of res.strongholds) {
        const feat = new Feature({ geometry: new Point([sh.x, sh.z]) })
        strongholdSource.addFeature(feat)
      }
    } catch (e) {
      console.warn('specials 失败:', e)
    }
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
      map.getView().setZoom(6)  // 默认 zoom 6 → 4 bpp
    }
    biomeSource?.refresh()
    biomeLayer?.setVisible(true)
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
      // 通知所有 Worker 切换种子（cubiomes_wrapper.c 每次 setup，但保留协议以兼容未来缓存）
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
    // 前往坐标时用 zoom 8（1 bpp，方块级精细）
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

  /** 关闭 popup（清空数据 + 隐藏 Overlay） */
  function closePopup(): void {
    popupData.value = null
    popupOverlay?.setPosition(undefined)
  }

  /** 前往结构坐标（动画居中 + 放大到 zoom 8） */
  function goToStruct(x: number, z: number): void {
    if (!map) return
    map.getView().animate({ center: [x, z], zoom: 8, duration: 500 })
  }

  /** 复制坐标到剪贴板（复用 format.ts 的 copyToClipboard） */
  async function copyCoord(x: number, z: number): Promise<void> {
    const text = formatCoord(x, z)
    const ok = await copyToClipboard(text)
    if (ok) toastSuccess('已复制: ' + text)
    else toastError('复制失败')
  }

  /**
   * 鼠标悬停查询群系名（debounce 300ms）
   *
   * 复用 WorkerPool.getBiomeAtPoint（scale=4 性能优于 scale=1）。
   * 失败时静默处理（鼠标悬停频繁，错误 toast 会刷屏）。
   */
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
    // 版本/维度切换后，清理已选但当前版本不可用的结构类型，避免筛选栏残留无效按钮
    // 排除 stronghold（由独立"要塞"按钮控制，不在结构列表中）
    const available = new Set(
      getStructuresForVersion(mcVersion.value, dimension.value)
        .filter(s => s.queryMode !== 'stronghold')
        .map(s => s.name),
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

  // ===== 特殊点显示控制 =====
  watch(showSpawn, (v) => { if (spawnLayer) spawnLayer.setVisible(v) })
  watch(showStronghold, (v) => { if (strongholdLayer) strongholdLayer.setVisible(v) })

  // ===== 地形渲染选项变化：Y 坐标 / 等高线 / 高度限制 =====
  watch([yCoord, doContour, ymaxLimit], () => {
    if (!currentSeed) return
    biomeSource?.refresh()
  })

  // ===== 生命周期 =====
  onMounted(async () => {
    // 必须先 await nextTick() 再 initMap，确保 popupContainer ref 已挂载到 DOM，
    // 否则 popupContainer.value 仍为 null，OL Overlay 创建时 element 为空。
    await nextTick()
    initMap()
    try {
      const wasmJsUrl = resUrl('cubiomes.js')
      const wasmUrl = resUrl('cubiomes.wasm')
      pool = new WorkerPool()
      await pool.init(wasmJsUrl, wasmUrl)
      // 自动加载默认种子 12345（不填入输入框，用户可自行输入新种子）
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

  // ===== 结构筛选 =====
  // 按 MC 版本 + 维度动态过滤可选结构（旧版本不显示新结构按钮）
  // 排除 queryMode='stronghold'：要塞由独立"要塞"按钮（showStronghold）控制，
  // 走 specials 流程（cubiomes_find_strongholds 多座迭代），避免在此列表重复显示且勾选无效。
  // queryMode='slime' 不被过滤：史莱姆区块在结构列表中勾选，由 findStructures 遍历 chunk。
  const structureListForVersion = computed(() => {
    return getStructuresForVersion(mcVersion.value, dimension.value)
      .filter(s => s.queryMode !== 'stronghold')
  })

  function toggleStructureType(name: string) {
    const next = new Set(selectedStructureTypes.value)
    if (next.has(name)) next.delete(name)
    else next.add(name)
    selectedStructureTypes.value = next
    renderStructures(structures.value)
  }

  function isStructureSelected(name: string): boolean {
    return selectedStructureTypes.value.has(name)
  }

  // 根据筛选条件渲染结构
  function renderStructures(items: WorkerStructure[]) {
    if (!structSource) return
    structSource.clear()
    const filtered = items.filter(s => selectedStructureTypes.value.has(s.stype))
    for (const st of filtered) {
      const feat = new Feature({
        geometry: new Point([st.x, st.z]),
        stype: st.stype,
        data: st,
      })
      feat.setStyle(getStructStyle(st.stype, false))
      structSource.addFeature(feat)
    }
  }

  return {
    seedInput, mcVersion, dimension, largeBiomes, userX, userZ,
    versionOptions, dimensionOptions,
    loading, structures, hoverStruct, mouseBlock, lastClickBlock,
    popupData, mouseBiomeName,
    mapContainer, popupContainer,
    showSpawn, showStronghold, showCoordPanel,
    yCoord, doContour, ymaxLimit,
    selectedStructureTypes, structureListForVersion,
    loadSeed, goToUserCoord, zoomIn, zoomOut, resetView,
    copyCoord, goToStruct, closePopup,
    getStructIcon, getStructIconUrl, toggleStructureType, isStructureSelected,
  }
}
