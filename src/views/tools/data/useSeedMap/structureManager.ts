/**
 * 种子地图结构管理（从 useSeedMap.ts 抽取）
 *
 * 通过工厂函数接收 composable 响应式状态与可变实例 getter，避免直接依赖闭包。
 * 负责结构/出生点/要塞的加载、渲染与筛选（详见各导出函数）。
 */

import { computed, type Ref } from 'vue'
import type OlMap from 'ol/Map'
import type VectorSource from 'ol/source/Vector'
import Feature from 'ol/Feature'
import Point from 'ol/geom/Point'
import { toastError } from '@/utils/toast'
import { WorkerPool } from '@/utils/seedmap/workerPool'
import { getStructuresForVersion } from '@/utils/seedmap/structures'
import type { Dimension, WorkerStructure } from '@/utils/seedmap/types'
import { STRUCT_MIN_ZOOM } from './config'

/** 结构管理所需的状态与实例 getter */
export interface StructureContext {
  // 响应式状态
  structures: Ref<WorkerStructure[]>
  selectedStructureTypes: Ref<Set<string>>
  showNonViable: Ref<boolean>
  mcVersion: Ref<number>
  dimension: Ref<Dimension>

  // 可变实例 getter（composable 中的 let 变量）
  getMap: () => OlMap | null
  getPool: () => WorkerPool | null
  getStructSource: () => VectorSource | null
  getSpawnSource: () => VectorSource | null
  getStrongholdSource: () => VectorSource | null
  getCurrentSeed: () => string
  getCurrentMc: () => number
  getCurrentDim: () => Dimension
  getCurrentLargeBiomes: () => boolean
}

/**
 * 创建结构管理器
 *
 * @param ctx 结构管理上下文（响应式状态 + 实例 getter）
 */
export function createStructureManager(ctx: StructureContext) {
  let structRequestId = 0
  /** 防止多个 findStructures 同时占用 Worker：上一次查找未完成时标记 pending */
  let structRefreshInProgress = false
  /** 查找期间有新请求到来时标记 pending，查找完成后自动补偿触发 */
  let structPendingRefresh = false

  /** 按 MC 版本 + 维度动态过滤可选结构（排除 stronghold，由独立按钮控制） */
  const structureListForVersion = computed(() => {
    return getStructuresForVersion(ctx.mcVersion.value, ctx.dimension.value)
      .filter(s => s.queryMode !== 'stronghold')
  })

  /** 加载结构（按可视范围） */
  async function refreshStructures() {
    const map = ctx.getMap()
    const pool = ctx.getPool()
    if (!ctx.getCurrentSeed() || !map || !pool) return
    const zoom = map.getView().getZoom() ?? 0
    if (zoom < STRUCT_MIN_ZOOM) {
      ctx.structures.value = []
      ctx.getStructSource()?.clear()
      return
    }
    if (structRefreshInProgress) {
      structPendingRefresh = true
      return
    }
    structRefreshInProgress = true
    const myId = ++structRequestId
    try {
      const view = map.getView()
      const extent = view.calculateExtent(map.getSize())
      const [minX, minZ, maxX, maxZ] = extent
      const margin = 256
      const items = await pool.findStructures({
        seed: ctx.getCurrentSeed(),
        mcVersion: ctx.getCurrentMc(),
        dimension: ctx.getCurrentDim(),
        largeBiomes: ctx.getCurrentLargeBiomes(),
        minX: Math.round(minX) - margin,
        minZ: Math.round(minZ) - margin,
        maxX: Math.round(maxX) + margin,
        maxZ: Math.round(maxZ) + margin,
      })
      if (myId !== structRequestId) return
      ctx.structures.value = items
      renderStructures(items)
    } catch (e) {
      toastError('结构加载失败: ' + (e instanceof Error ? e.message : String(e)))
    } finally {
      structRefreshInProgress = false
      if (structPendingRefresh) {
        structPendingRefresh = false
        setTimeout(() => refreshStructures(), 0)
      }
    }
  }

  /** 加载特殊点（出生点/多座要塞） */
  async function refreshSpecials() {
    const spawnSource = ctx.getSpawnSource()
    const strongholdSource = ctx.getStrongholdSource()
    const pool = ctx.getPool()
    if (!ctx.getCurrentSeed() || !spawnSource || !strongholdSource || !pool) return
    try {
      const res = await pool.getSpecials({
        seed: ctx.getCurrentSeed(),
        mcVersion: ctx.getCurrentMc(),
        largeBiomes: ctx.getCurrentLargeBiomes(),
      })
      spawnSource.clear()
      if (res.spawn) {
        const feat = new Feature({ geometry: new Point([res.spawn.x, res.spawn.z]) })
        spawnSource.addFeature(feat)
      }
      strongholdSource.clear()
      for (const sh of res.strongholds) {
        const feat = new Feature({ geometry: new Point([sh.x, sh.z]) })
        strongholdSource.addFeature(feat)
      }
    } catch (e) {
      console.warn('specials 失败:', e)
      toastError('加载 specials 失败')
    }
  }

  /** 根据筛选条件渲染结构 Feature */
  function renderStructures(items: WorkerStructure[]) {
    const structSource = ctx.getStructSource()
    if (!structSource) return
    structSource.clear()
    const filtered = items.filter(s =>
      ctx.selectedStructureTypes.value.has(s.stype)
      && (ctx.showNonViable.value || s.viable),
    )
    for (const st of filtered) {
      const feat = new Feature({
        geometry: new Point([st.x, st.z]),
        stype: st.stype,
        data: st,
      })
      structSource.addFeature(feat)
    }
  }

  function toggleStructureType(name: string) {
    const next = new Set(ctx.selectedStructureTypes.value)
    if (next.has(name)) next.delete(name)
    else next.add(name)
    ctx.selectedStructureTypes.value = next
    renderStructures(ctx.structures.value)
  }

  function isStructureSelected(name: string): boolean {
    return ctx.selectedStructureTypes.value.has(name)
  }

  return {
    structureListForVersion,
    refreshStructures,
    refreshSpecials,
    renderStructures,
    toggleStructureType,
    isStructureSelected,
  }
}
