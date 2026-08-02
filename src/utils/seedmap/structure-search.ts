/**
 * 种子地图结构查找
 *
 * 结构定位（region 遍历 / slime / ravine / fossil）+ 出生点 + 多座要塞 + 单点群系查询。
 * 含各类 chunk/region 遍历上限，防止大范围缩放时卡死 Worker。
 */
import { getStructuresByDimension } from './structures'
import type { Dimension, WorkerStructure } from './types'
import { ensureHeap, wasm, writeSeedString } from './wasm-bindings'

/** 史莱姆区块遍历上限：可视范围 chunk 数超过此值时跳过 */
const SLIME_CHUNK_LIMIT = 10000

/** 峡谷/化石查找的 chunk 范围硬上限（非 mega 64x64=4096 chunks；mega 32x32=1024 chunks） */
const CHUNK_FIND_LIMIT_NON_MEGA = 64
const CHUNK_FIND_LIMIT_MEGA = 32

/** carveCanyon poses.size 阈值：超过此值视为 mega 峡谷 */
const MEGA_RAVINE_POSE_THRESHOLD = 200

/** region 遍历总数上限（5000 region ≈ 70×70 region = 36K×36K 方块） */
const REGION_TRAVERSE_LIMIT = 5000

/** 要塞查询上限（cubiomes 默认要塞总数） */
const MAX_STRONGHOLDS = 128

export interface FindStructuresParams {
  seed: string
  mcVersion: number
  dimension: Dimension
  largeBiomes: boolean
  minX: number
  minZ: number
  maxX: number
  maxZ: number
}

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

/**
 * 调用 cubiomes chunk 范围查找函数（ravines/nether_fossils/fossils 共用模式）
 *
 * 统一封装：chunk 范围计算 → buffer 分配 → WASM 调用 → 结果读取 → buffer 释放。
 * 可视范围超过 sizeLimit 时自动分块查找，避免缩放时结构被整片跳过。
 */
function callChunkFinder(
  fn: (startCX: number, startCZ: number, numX: number, numZ: number,
       bufPtr: number, bufLen: number) => number,
  minX: number, minZ: number, maxX: number, maxZ: number,
  sizeLimit: number,
): { x: number; z: number }[] | null {
  const startCX = Math.floor(minX / 16)
  const endCX = Math.floor(maxX / 16)
  const startCZ = Math.floor(minZ / 16)
  const endCZ = Math.floor(maxZ / 16)
  const totalX = endCX - startCX + 1
  const totalZ = endCZ - startCZ + 1

  if (totalX <= sizeLimit && totalZ <= sizeLimit) {
    return callFinderOnce(fn, startCX, startCZ, totalX, totalZ)
  }

  const merged: { x: number; z: number }[] = []
  for (let sx = 0; sx < totalX; sx += sizeLimit) {
    for (let sz = 0; sz < totalZ; sz += sizeLimit) {
      const sub = callFinderOnce(fn, startCX + sx, startCZ + sz,
        Math.min(sizeLimit, totalX - sx), Math.min(sizeLimit, totalZ - sz))
      if (sub === null) return null
      for (const r of sub) merged.push(r)
    }
  }
  return merged
}

/** 单次 WASM chunk finder 调用（范围已确保 ≤ sizeLimit） */
function callFinderOnce(
  fn: (startCX: number, startCZ: number, numX: number, numZ: number,
       bufPtr: number, bufLen: number) => number,
  startCX: number, startCZ: number, numX: number, numZ: number,
): { x: number; z: number }[] | null {
  const bufLen = numX * numZ * 2
  const bufPtr = wasm.module._malloc(bufLen * 4)
  if (!bufPtr) return null

  try {
    const count = fn(startCX, startCZ, numX, numZ, bufPtr, bufLen)
    if (count <= 0) return []
    const heap = ensureHeap()
    const buf = new Int32Array(heap.buffer, bufPtr, count * 2)
    const result: { x: number; z: number }[] = []
    for (let i = 0; i < count; i++) {
      result.push({ x: buf[i * 2], z: buf[i * 2 + 1] })
    }
    return result
  } finally {
    wasm.module._free(bufPtr)
  }
}

/** 结构查找（slime / ravine / fossil / region 遍历四类） */
export function findStructures(msg: FindStructuresParams): WorkerStructure[] {
  const { seed, mcVersion, dimension, largeBiomes, minX, minZ, maxX, maxZ } = msg
  const structs: WorkerStructure[] = []
  const types = getStructuresByDimension(dimension)

  const seedPtr = writeSeedString(seed)
  try {
    for (const tconf of types) {
      if (tconf.queryMode === 'stronghold') continue

      if (tconf.queryMode === 'slime') {
        try {
          const startCX = Math.floor(minX / 16)
          const endCX = Math.floor(maxX / 16)
          const startCZ = Math.floor(minZ / 16)
          const endCZ = Math.floor(maxZ / 16)
          const chunkCount = (endCX - startCX + 1) * (endCZ - startCZ + 1)
          if (chunkCount > SLIME_CHUNK_LIMIT) continue

          for (let cx = startCX; cx <= endCX; cx++) {
            for (let cz = startCZ; cz <= endCZ; cz++) {
              if (wasm.module._cubiomes_is_slime_chunk(seedPtr, cx, cz)) {
                structs.push({
                  stype: tconf.name,
                  x: cx * 16 + 8,
                  z: cz * 16 + 8,
                  viable: true,
                })
              }
            }
          }
        } catch { /* isSlimeChunk 调用失败，跳过 */ }
        continue
      }

      const qm = tconf.queryMode
      if (qm === 'ravine' || qm === 'mega_ravine'
          || qm === 'underwater_ravine' || qm === 'mega_underwater_ravine') {
        const isUnderwater = (qm === 'underwater_ravine' || qm === 'mega_underwater_ravine')
        const isMega = (qm === 'mega_ravine' || qm === 'mega_underwater_ravine')
        const canyonType = isUnderwater ? 1 : 0
        const megaThreshold = isMega ? MEGA_RAVINE_POSE_THRESHOLD : 0
        const sizeLimit = isMega ? CHUNK_FIND_LIMIT_MEGA : CHUNK_FIND_LIMIT_NON_MEGA

        try {
          const results = callChunkFinder(
            (sx, sz, nx, nz, bp, bl) => wasm.module._cubiomes_find_ravines(
              seedPtr, mcVersion, dimension, sx, sz, nx, nz,
              canyonType, megaThreshold, bp, bl,
            ),
            minX, minZ, maxX, maxZ, sizeLimit,
          )
          if (results) {
            for (const r of results) {
              structs.push({ stype: tconf.name, x: r.x, z: r.z, viable: true })
            }
          }
        } catch { /* ravine 查找失败，跳过 */ }
        continue
      }

      if (qm === 'nether_fossil') {
        try {
          const results = callChunkFinder(
            (sx, sz, nx, nz, bp, bl) => wasm.module._cubiomes_find_nether_fossils(
              seedPtr, mcVersion, sx, sz, nx, nz, bp, bl,
            ),
            minX, minZ, maxX, maxZ, CHUNK_FIND_LIMIT_NON_MEGA,
          )
          if (results) {
            for (const r of results) {
              structs.push({ stype: tconf.name, x: r.x, z: r.z, viable: true })
            }
          }
        } catch { /* nether_fossil 查找失败，跳过 */ }
        continue
      }

      if (qm === 'fossil' || qm === 'fossil_diamond') {
        const diamondMode = qm === 'fossil_diamond' ? 1 : 0
        try {
          const results = callChunkFinder(
            (sx, sz, nx, nz, bp, bl) => wasm.module._cubiomes_find_fossils(
              seedPtr, mcVersion, dimension, sx, sz, nx, nz,
              diamondMode, bp, bl,
            ),
            minX, minZ, maxX, maxZ, CHUNK_FIND_LIMIT_NON_MEGA,
          )
          if (results) {
            for (const r of results) {
              structs.push({ stype: tconf.name, x: r.x, z: r.z, viable: true })
            }
          }
        } catch { /* fossil 查找失败，跳过 */ }
        continue
      }

      try {
        const regionSize = wasm.module._cubiomes_get_region_size(tconf.id, mcVersion)
        if (!regionSize) continue
        const regionSizeBlocks = regionSize * 16

        const startRegX = Math.floor(minX / regionSizeBlocks)
        const endRegX = Math.floor(maxX / regionSizeBlocks)
        const startRegZ = Math.floor(minZ / regionSizeBlocks)
        const endRegZ = Math.floor(maxZ / regionSizeBlocks)

        const regionCount = (endRegX - startRegX + 1) * (endRegZ - startRegZ + 1)
        if (regionCount > REGION_TRAVERSE_LIMIT) continue

        for (let rx = startRegX; rx <= endRegX; rx++) {
          for (let rz = startRegZ; rz <= endRegZ; rz++) {
            const outXPtr = wasm.module._malloc(4)
            const outZPtr = wasm.module._malloc(4)
            if (!outXPtr || !outZPtr) {
              if (outXPtr) wasm.module._free(outXPtr)
              if (outZPtr) wasm.module._free(outZPtr)
              continue
            }
            try {
              const found = wasm.module._cubiomes_get_structure_pos(
                tconf.id, seedPtr, mcVersion, rx, rz, outXPtr, outZPtr,
              )
              if (found) {
                const heap = ensureHeap()
                const x = new Int32Array(heap.buffer, outXPtr, 1)[0]
                const z = new Int32Array(heap.buffer, outZPtr, 1)[0]
                if (x >= minX && x <= maxX && z >= minZ && z <= maxZ) {
                  let viable = true
                  try {
                    viable = wasm.module._cubiomes_is_viable(
                      tconf.id, seedPtr, mcVersion, dimension, largeBiomes ? 1 : 0, x, z,
                    ) === 1
                  } catch { /* skip viability check */ }
                  structs.push({ stype: tconf.name, x, z, viable })
                }
              }
            } finally {
              wasm.module._free(outXPtr)
              wasm.module._free(outZPtr)
            }
          }
        }
      } catch { /* 该结构类型可能不支持，跳过 */ }
    }
  } finally {
    wasm.module._free(seedPtr)
  }

  return structs
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
