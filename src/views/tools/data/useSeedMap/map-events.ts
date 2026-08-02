/**
 * 种子地图事件处理
 *
 * 从 useSeedMap.ts 抽取的地图事件处理逻辑（pointermove / singleclick 几何命中检测、
 * 坐标显示、hover 高亮、moveend 结构刷新调度、悬停群系名查询）。
 * 通过工厂函数接收响应式状态 refs、可变实例 getter 与回调，避免直接依赖闭包。
 *
 * 返回 dispose 函数，用于清理内部定时器（onBeforeUnmount 调用）。
 */

import type { Ref } from 'vue'
import type OlMap from 'ol/Map'
import type VectorSource from 'ol/source/Vector'
import type Overlay from 'ol/Overlay'
import Feature from 'ol/Feature'
import Point from 'ol/geom/Point'
import type { WorkerPool } from '@/utils/seedmap/workerPool'
import { getStructIcon } from '@/utils/seedmap/constants'
import { getBiomeName } from '@/utils/seedmap/biomeNames'
import type { Dimension, WorkerStructure } from '@/utils/seedmap/types'

/** 事件处理所需的状态与实例 getter */
export interface MapEventsContext {
  // 响应式状态
  hoverStruct: Ref<WorkerStructure | null>
  hoverMarker: Ref<{ label: string; x: number; z: number } | null>
  mouseBlock: Ref<{ x: number; z: number } | null>
  lastClickBlock: Ref<{ x: number; z: number } | null>
  popupData: Ref<{ struct: WorkerStructure; coord: [number, number] } | null>
  mouseBiomeName: Ref<string>
  showSpawn: Ref<boolean>
  showStronghold: Ref<boolean>
  dimension: Ref<Dimension>
  yCoord: Ref<number>
  /** 结构高亮 Feature 引用（与 style 闭包共享） */
  highlight: { hoverFeat: Feature | null; clickFeat: Feature | null }

  // 可变实例 getter
  getMap: () => OlMap | null
  getStructSource: () => VectorSource | null
  getSpawnSource: () => VectorSource | null
  getStrongholdSource: () => VectorSource | null
  getClickMarkerSource: () => VectorSource | null
  getPopupOverlay: () => Overlay | null

  // 当前种子快照（悬停群系查询用）
  getCurrentSeed: () => string
  getCurrentMc: () => number
  getCurrentDim: () => Dimension
  getCurrentLargeBiomes: () => boolean
  getPool: () => WorkerPool | null

  // moveend 回调
  updateStructLayerVisibility: () => void
  refreshStructures: () => void
  // 关闭 popup
  closePopup: () => void
}

/**
 * 注册种子地图事件（singleclick / pointermove / moveend）
 *
 * @param ctx 事件处理上下文
 * @returns dispose 函数（清理内部定时器）
 */
export function createMapEvents(ctx: MapEventsContext): () => void {
  const {
    hoverStruct, hoverMarker, mouseBlock, lastClickBlock, popupData, mouseBiomeName,
    showSpawn, showStronghold, dimension, yCoord, highlight,
  } = ctx

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
    const map = ctx.getMap()
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
    checkSource(ctx.getStructSource(), 'struct', true)
    checkSource(ctx.getSpawnSource(), 'spawn', showSpawn.value)
    checkSource(ctx.getStrongholdSource(), 'stronghold', showStronghold.value && dimension.value === 0)
    return best
  }

  /** 鼠标悬停查询群系名（debounce 300ms） */
  let biomeDebounce: number | null = null
  function scheduleBiomeQuery(blockX: number, blockZ: number): void {
    if (biomeDebounce) clearTimeout(biomeDebounce)
    biomeDebounce = window.setTimeout(async () => {
      biomeDebounce = null
      const pool = ctx.getPool()
      if (!ctx.getCurrentSeed() || !pool) {
        mouseBiomeName.value = ''
        return
      }
      try {
        const biomeId = await pool.getBiomeAtPoint({
          seed: ctx.getCurrentSeed(),
          mcVersion: ctx.getCurrentMc(),
          dimension: ctx.getCurrentDim(),
          largeBiomes: ctx.getCurrentLargeBiomes(),
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

  const map = ctx.getMap()
  if (!map) return () => {}

  let hoverThrottle: number | null = null
  let lastPixel: number[] | null = null

  map.on('singleclick', (e) => {
    const hit = findFeatureAtPixel(e.pixel)
    if (hit) {
      if (hit.type === 'struct') {
        highlight.clickFeat = hit.feature
        const struct = hit.feature.get('data') as WorkerStructure
        popupData.value = { struct, coord: [struct.x, struct.z] }
        ctx.getPopupOverlay()?.setPosition([struct.x, struct.z])
        ctx.getStructSource()!.changed()
      } else {
        highlight.clickFeat = null
        ctx.closePopup()
        lastClickBlock.value = { x: hit.x, z: hit.z }
        ctx.getClickMarkerSource()!.clear()
        ctx.getClickMarkerSource()!.addFeature(new Feature({ geometry: new Point([hit.x, hit.z]) }))
      }
    } else {
      highlight.clickFeat = null
      ctx.closePopup()
      const [cx, cz] = e.coordinate
      lastClickBlock.value = { x: Math.round(cx), z: Math.round(cz) }
      ctx.getClickMarkerSource()!.clear()
      ctx.getClickMarkerSource()!.addFeature(new Feature({ geometry: new Point([cx, cz]) }))
    }
  })

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
          if (hit.feature !== highlight.hoverFeat) {
            highlight.hoverFeat = hit.feature
            hoverStruct.value = hit.feature.get('data') as WorkerStructure
            hoverMarker.value = null
            ctx.getStructSource()!.changed()
          }
        } else {
          if (highlight.hoverFeat !== null) {
            highlight.hoverFeat = null
            hoverStruct.value = null
            ctx.getStructSource()!.changed()
          }
          hoverMarker.value = { label: hit.label, x: hit.x, z: hit.z }
        }
      } else {
        if (highlight.hoverFeat !== null) {
          highlight.hoverFeat = null
          hoverStruct.value = null
          ctx.getStructSource()!.changed()
        }
        hoverMarker.value = null
      }
    }, 50)
    scheduleBiomeQuery(blockX, blockZ)
  })

  let moveendTimer: number | null = null
  map.on('moveend', () => {
    ctx.updateStructLayerVisibility()
    if (!ctx.getCurrentSeed()) return
    if (moveendTimer) clearTimeout(moveendTimer)
    moveendTimer = window.setTimeout(() => {
      moveendTimer = null
      ctx.refreshStructures()
    }, 300)
  })

  return () => {
    if (biomeDebounce) {
      clearTimeout(biomeDebounce)
      biomeDebounce = null
    }
    if (hoverThrottle) {
      clearTimeout(hoverThrottle)
      hoverThrottle = null
    }
    if (moveendTimer) {
      clearTimeout(moveendTimer)
      moveendTimer = null
    }
  }
}
