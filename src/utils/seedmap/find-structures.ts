import { getStructuresByDimension } from './structures'
import type { Dimension, WorkerStructure } from './types'
import { ensureHeap, wasm, writeSeedString } from './wasm-bindings'
import { callChunkFinder, CHUNK_FIND_LIMIT_MEGA, CHUNK_FIND_LIMIT_NON_MEGA, MEGA_RAVINE_POSE_THRESHOLD } from './chunk-finder'

/** 史莱姆区块遍历上限：可视范围 chunk 数超过此值时跳过 */
const SLIME_CHUNK_LIMIT = 10000

/** region 遍历总数上限（5000 region ≈ 70×70 region = 36K×36K 方块） */
const REGION_TRAVERSE_LIMIT = 5000

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