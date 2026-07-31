/**
 * 种子地图 tile 加载器
 *
 * 从 useSeedMap.ts 抽取的群系 tile 加载逻辑。通过工厂函数接收当前种子/版本等
 * 状态的 getter，避免直接依赖 composable 闭包。
 *
 * 坐标系约定（与 generatorWorker.ts 对齐）：
 * - OL TileGrid 用 top-left origin = [-EXTENT_HALF, +EXTENT_HALF]
 * - tile y=0 在屏幕顶部 = 投影 max Y = MC max Z（本项目约定 +Z=北方）
 * - startBlockZ 取 tile 的 min Z（南方边缘），由 worker 翻转 Z 轴后渲染
 * - blockX/blockZ 始终是 **方块坐标**；worker 内部负责 block → scale 坐标转换
 */

import { toastError } from '@/utils/toast'
import { WorkerPool } from '@/utils/seedmap/workerPool'
import type { Dimension } from '@/utils/seedmap/types'
import { TILE_SIZE, EXTENT_HALF, RESOLUTIONS } from './config'

/** tile 加载器所需的状态快照 */
export interface TileLoaderState {
  seed: string
  mcVersion: number
  dimension: Dimension
  largeBiomes: boolean
  yCoord: number
  doContour: boolean
  ymaxLimit: number
  pool: WorkerPool | null
}

/**
 * tile 加载失败 toast 防抖标志：
 * 首次失败时置 true 并 toast，成功后重置为 false，避免连续失败刷屏。
 */
let tileErrorToastShown = false

/** 创建空 bitmap（extent 外或无种子时返回） */
async function emptyBitmap(): Promise<ImageBitmap> {
  const c = document.createElement('canvas')
  c.width = c.height = TILE_SIZE
  return createImageBitmap(c)
}

/**
 * 创建 tile 加载器
 *
 * @param getState 返回当前 tile 加载所需状态的函数（每次 tile 请求时调用）
 * @returns OL DataTile loader 函数 (z, x, y) => Promise<ImageBitmap>
 */
export function createTileLoader(getState: () => TileLoaderState) {
  return async function loadBiomeTile(z: number, x: number, y: number): Promise<ImageBitmap> {
    const state = getState()
    if (!state.seed || !state.pool) return emptyBitmap()
    // z 直接对应 RESOLUTIONS 索引（OL zoom 0~10）
    const res = RESOLUTIONS[z]
    if (!res) return emptyBitmap()
    const bpp = res // 方块/像素
    // cubiomes 只支持 scale ∈ {1, 4, 16, 64, 256}
    const SUPPORTED_SCALES = [1, 4, 16, 64, 256]
    const scale: number = SUPPORTED_SCALES.filter(s => s <= bpp).pop() ?? 1
    const sx = Math.min(2048, Math.max(1, Math.round((TILE_SIZE * bpp) / scale)))
    const sz = Math.min(2048, Math.max(1, Math.round((TILE_SIZE * bpp) / scale)))
    const blocksPerTile = TILE_SIZE * bpp
    const startBlockX = Math.round(-EXTENT_HALF + x * blocksPerTile)
    const startBlockZ = Math.round(EXTENT_HALF - (y + 1) * blocksPerTile)
    // 防御性边界检查：跳过 extent 范围外的 tile
    if (startBlockX + blocksPerTile <= -EXTENT_HALF || startBlockX >= EXTENT_HALF ||
        startBlockZ + blocksPerTile <= -EXTENT_HALF || startBlockZ >= EXTENT_HALF) {
      return emptyBitmap()
    }
    const tileParams = {
      seed: state.seed,
      mcVersion: state.mcVersion,
      dimension: state.dimension,
      largeBiomes: state.largeBiomes,
      blockX: startBlockX,
      blockZ: startBlockZ,
      sx, sz, scale,
      y: state.yCoord,
      doContour: state.doContour,
      ymax: state.ymaxLimit > 0 ? state.ymaxLimit : Infinity,
    }
    // 重试机制：Worker 可能因 WASM 初始化未完成或内存增长偶发失败
    const MAX_RETRIES = 2
    for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
      try {
        const bitmap = await state.pool.generateTile(tileParams)
        tileErrorToastShown = false
        return bitmap
      } catch (e) {
        if (attempt < MAX_RETRIES) {
          await new Promise(r => setTimeout(r, 200))
          continue
        }
        console.error('[seedmap] tile load failed after retries', { z, x, y, error: e instanceof Error ? e.message : String(e) })
        if (!tileErrorToastShown) {
          tileErrorToastShown = true
          toastError('地图加载失败，请重试')
        }
        return emptyBitmap()
      }
    }
    return emptyBitmap()
  }
}
