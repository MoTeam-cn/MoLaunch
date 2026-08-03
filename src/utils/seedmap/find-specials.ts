import type { Dimension } from './types'
import { ensureHeap, wasm, writeSeedString } from './wasm-bindings'

/** 要塞查询上限（cubiomes 默认要塞总数） */
const MAX_STRONGHOLDS = 128

export interface SpecialsParams {
  seed: string
  mcVersion: number
  largeBiomes: boolean
}

export interface BiomeAtPointParams {
  seed: string
  mcVersion: number
  dimension: Dimension
  largeBiomes: boolean
  scale: number
  x: number
  y: number
  z: number
}

/** 出生点 + 多座要塞（cubiomes_find_strongholds 一次性返回，OL 自动渲染所有 Feature） */
export function findSpecials(msg: SpecialsParams): { spawn: { x: number; z: number } | null; strongholds: { x: number; z: number }[] } {
  const { seed, mcVersion, largeBiomes } = msg
  const seedPtr = writeSeedString(seed)

  let spawn: { x: number; z: number } | null = null
  const strongholds: { x: number; z: number }[] = []

  try {
    const spawnXPtr = wasm.module._malloc(4)
    const spawnZPtr = wasm.module._malloc(4)
    if (spawnXPtr && spawnZPtr) {
      try {
        wasm.module._cubiomes_estimate_spawn(seedPtr, mcVersion, largeBiomes ? 1 : 0, spawnXPtr, spawnZPtr)
        const heap = ensureHeap()
        spawn = {
          x: new Int32Array(heap.buffer, spawnXPtr, 1)[0],
          z: new Int32Array(heap.buffer, spawnZPtr, 1)[0],
        }
      } finally {
        wasm.module._free(spawnXPtr)
        wasm.module._free(spawnZPtr)
      }
    }

    const bufLen = MAX_STRONGHOLDS * 2
    const shBufPtr = wasm.module._malloc(bufLen * 4)
    if (shBufPtr) {
      try {
        const count = wasm.module._cubiomes_find_strongholds(
          seedPtr, mcVersion, MAX_STRONGHOLDS, shBufPtr, bufLen,
        )
        if (count > 0) {
          const heap = ensureHeap()
          const buf = new Int32Array(heap.buffer, shBufPtr, count * 2)
          for (let i = 0; i < count; i++) {
            strongholds.push({ x: buf[i * 2], z: buf[i * 2 + 1] })
          }
        }
      } catch { /* 要塞查找失败 */ } finally {
        wasm.module._free(shBufPtr)
      }
    }
  } finally {
    wasm.module._free(seedPtr)
  }

  return { spawn, strongholds }
}

/** 单点群系查询（鼠标悬停显示群系名） */
export function findBiomeAtPoint(msg: BiomeAtPointParams): number {
  const seedPtr = writeSeedString(msg.seed)
  try {
    return wasm.module._cubiomes_get_biome_at_point(
      seedPtr, msg.mcVersion, msg.dimension, msg.largeBiomes ? 1 : 0,
      msg.scale, msg.x, msg.y, msg.z,
    )
  } finally {
    wasm.module._free(seedPtr)
  }
}