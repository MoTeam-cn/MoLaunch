/**
 * 种子地图 tile 渲染
 *
 * biome 上色 + 地形阴影 + createImageBitmap 位图生成，从 generatorWorker 拆出。
 * 坐标系约定：OL TileGrid top-left origin，tile 顶部 = MC max Z（+Z=北方），图像 py=0 需对应 cubiomes gz=sz-1。
 */
import { BIOME_COLORS, DEFAULT_COLOR } from './constants'
import { applyTerrainShading } from './terrainShading'
import { ensureHeap, getWasmBiomeColors, wasm } from './wasm-bindings'

/** 单 tile 像素尺寸（与 cubiomes_viewer 一致） */
export const TILE_SIZE = 64

export interface RenderTileParams {
  seedPtr: number
  mcVersion: number
  dimension: number
  largeBiomes: boolean
  scale: number
  blockX: number
  blockZ: number
  sx: number
  sz: number
  y?: number
  doContour?: boolean
  ymax?: number
}

/**
 * 生成单 tile 位图（biome 上色 + 可选地形阴影）
 *
 * scale > 16 时跳过高度生成（cubiomes 内部 1:4 分辨率导致 height buffer 膨胀），
 * 远观级别阴影细节本就不可见。
 */
export async function renderTile(p: RenderTileParams): Promise<ImageBitmap> {
  const y = p.y ?? 64
  if (p.scale <= 0 || !Number.isInteger(p.scale)) {
    throw new Error(`invalid scale=${p.scale}（必须为正整数）`)
  }
  if (p.blockX % p.scale !== 0 || p.blockZ % p.scale !== 0) {
    throw new Error(
      `blockX/Z(${p.blockX},${p.blockZ}) 必须是 scale(${p.scale}) 的整数倍，` +
      `否则 tile 边界无法对齐 scale 网格`,
    )
  }
  const rangeX = (p.blockX / p.scale) | 0
  const rangeZ = (p.blockZ / p.scale) | 0

  const SKIP_HEIGHT_THRESHOLD = 16
  const withHeight = p.scale <= SKIP_HEIGHT_THRESHOLD

  const ret = withHeight
    ? wasm.module._cubiomes_gen_biomes_with_height_static(
        p.seedPtr, p.mcVersion, p.dimension, p.largeBiomes ? 1 : 0,
        p.scale, rangeX, rangeZ, p.sx, p.sz, y,
      )
    : wasm.module._cubiomes_gen_biomes_static(
        p.seedPtr, p.mcVersion, p.dimension, p.largeBiomes ? 1 : 0,
        p.scale, rangeX, rangeZ, p.sx, p.sz,
      )
  if (ret !== 0) {
    throw new Error(
      withHeight
        ? `_cubiomes_gen_biomes_with_height_static 失败 (code=${ret})`
        : `_cubiomes_gen_biomes_static 失败 (code=${ret})`,
    )
  }

  const heap = ensureHeap()
  const biomePtr = wasm.module._cubiomes_get_biome_data_pointer()
  const biomeSize = wasm.module._cubiomes_get_biome_data_size()
  if (!biomePtr || biomeSize < p.sx * p.sz) {
    throw new Error(`biome buffer 无效 (ptr=${biomePtr}, size=${biomeSize}, expect=${p.sx * p.sz})`)
  }
  const biomeData = new Int32Array(heap.buffer, biomePtr, p.sx * p.sz)

  const rgba = new Uint8ClampedArray(TILE_SIZE * TILE_SIZE * 4)
  const wasmBiomeColors = getWasmBiomeColors()
  for (let py = 0; py < TILE_SIZE; py++) {
    const gz = p.sz - 1 - Math.min(p.sz - 1, Math.floor(py * p.sz / TILE_SIZE))
    for (let px = 0; px < TILE_SIZE; px++) {
      const gx = Math.min(p.sx - 1, Math.floor(px * p.sx / TILE_SIZE))
      const id = biomeData[gz * p.sx + gx]
      const idx = (py * TILE_SIZE + px) * 4
      if (wasmBiomeColors && id >= 0 && id < 256) {
        const o = id * 3
        rgba[idx] = wasmBiomeColors[o]
        rgba[idx + 1] = wasmBiomeColors[o + 1]
        rgba[idx + 2] = wasmBiomeColors[o + 2]
      } else {
        const c = BIOME_COLORS[id] ?? DEFAULT_COLOR
        rgba[idx] = c[0]
        rgba[idx + 1] = c[1]
        rgba[idx + 2] = c[2]
      }
      rgba[idx + 3] = 255
    }
  }

  if (withHeight) {
    const heap2 = ensureHeap()
    const dimsPtr = wasm.module._cubiomes_get_height_grid_dims()
    const dims = new Int32Array(heap2.buffer, dimsPtr, 2)
    const hw = dims[0]
    const hh = dims[1]
    if (hw > 0 && hh > 0) {
      const heightPtr = wasm.module._cubiomes_get_height_data_pointer()
      if (heightPtr) {
        const heightData = new Float32Array(heap2.buffer, heightPtr, hw * hh)
        const heightCellPx = TILE_SIZE / hw
        const heights = new Float32Array(hw * hh)
        for (let z = 0; z < hh; z++) {
          const srcZ = hh - 1 - z
          heights.set(heightData.subarray(srcZ * hw, (srcZ + 1) * hw), z * hw)
        }
        applyTerrainShading(rgba, TILE_SIZE, TILE_SIZE, heights, hw, hh, heightCellPx, {
          scale: p.scale,
          pixelsPerCell: TILE_SIZE / p.sx,
          doContour: p.doContour ?? false,
          ymax: p.ymax ?? Infinity,
        })
      }
    }
  }

  const imageData = new ImageData(rgba, TILE_SIZE, TILE_SIZE)
  return createImageBitmap(imageData)
}
