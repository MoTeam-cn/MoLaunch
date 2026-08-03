/**
 * 种子地图 OL Map 与图层初始化（从 useSeedMap.ts 抽取）
 *
 * 通过工厂函数接收 tile loader、popup 与结构高亮 Feature，避免依赖闭包。
 * 负责 'mc' 投影、群系 DataTile / 结构 Vector 图层，以及 Map/View/Overlay/交互。
 */

import OlMap from 'ol/Map'
import View from 'ol/View'
import Projection from 'ol/proj/Projection'
import DataTile, { type Loader as DataTileLoader } from 'ol/source/DataTile'
import TileLayer from 'ol/layer/Tile'
import VectorLayer from 'ol/layer/Vector'
import VectorSource from 'ol/source/Vector'
import type Feature from 'ol/Feature'
import TileGrid from 'ol/tilegrid/TileGrid'
import Overlay from 'ol/Overlay'
import { defaults as defaultInteractions } from 'ol/interaction'
import MouseWheelZoom from 'ol/interaction/MouseWheelZoom'
import { getStructStyle, getMarkerStyle, getClickMarkerStyle } from '@/utils/seedmap/constants'
import { MIN_ZOOM, MAX_ZOOM, TILE_SIZE, RESOLUTIONS, EXTENT, EXTENT_HALF } from './config'

/** 地图初始化所需的外部依赖 */
export interface SeedMapInitOptions {
  /** OL Map 挂载容器 */
  target: HTMLElement
  /** 群系 tile loader */
  loader: DataTileLoader
  /** popup 浮窗容器（为空则不创建 Overlay） */
  popupContainer: HTMLElement | null
  /** 结构高亮 Feature 引用（style 闭包与事件处理共享） */
  highlight: { hoverFeat: Feature | null; clickFeat: Feature | null }
}

/** 地图初始化产物（图层/资源实例，由调用方持有） */
export interface SeedMapInitResult {
  map: OlMap
  biomeSource: DataTile
  biomeLayer: TileLayer
  structSource: VectorSource
  structLayer: VectorLayer
  spawnSource: VectorSource
  spawnLayer: VectorLayer
  strongholdSource: VectorSource
  strongholdLayer: VectorLayer
  clickMarkerSource: VectorSource
  clickLayer: VectorLayer
  popupOverlay: Overlay | null
}

/**
 * 初始化种子地图（投影 / 图层 / 控件 / 交互）
 *
 * @param opts 初始化选项
 */
export function initSeedMap(opts: SeedMapInitOptions): SeedMapInitResult {
  const { loader, popupContainer, highlight } = opts

  const projection = new Projection({
    code: 'mc',
    units: 'm',
    extent: EXTENT,
  })

  const biomeSource = new DataTile({
    tileGrid: new TileGrid({
      extent: EXTENT,
      origin: [-EXTENT_HALF, EXTENT_HALF],
      resolutions: RESOLUTIONS,
      tileSize: TILE_SIZE,
    }),
    projection,
    loader,
    wrapX: false,
    transition: 0,
  })
  const biomeLayer = new TileLayer({ source: biomeSource, visible: false, cacheSize: 4096, preload: 1 })

  const structSource = new VectorSource()
  const structLayer = new VectorLayer({
    source: structSource,
    style: (feature) => {
      const stype = feature.get('stype') as string
      const highlighted = feature === highlight.hoverFeat || feature === highlight.clickFeat
      return getStructStyle(stype, highlighted)
    },
  })

  const spawnSource = new VectorSource()
  const spawnLayer = new VectorLayer({ source: spawnSource, style: getMarkerStyle('#4CAF50') })

  const strongholdSource = new VectorSource()
  const strongholdLayer = new VectorLayer({ source: strongholdSource, style: getMarkerStyle('#E91E63') })

  const clickMarkerSource = new VectorSource()
  const clickLayer = new VectorLayer({
    source: clickMarkerSource,
    style: getClickMarkerStyle(),
  })

  const popupOverlay = popupContainer
    ? new Overlay({
        element: popupContainer,
        autoPan: { animation: { duration: 250 }, margin: 16 },
        offset: [0, -16],
        positioning: 'bottom-center',
      })
    : null

  const map = new OlMap({
    target: opts.target,
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

  return {
    map, biomeSource, biomeLayer,
    structSource, structLayer,
    spawnSource, spawnLayer,
    strongholdSource, strongholdLayer,
    clickMarkerSource, clickLayer,
    popupOverlay,
  }
}
