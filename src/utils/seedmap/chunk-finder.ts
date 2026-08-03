/** 峡谷/化石查找的 chunk 范围硬上限（非 mega 64x64=4096 chunks；mega 32x32=1024 chunks） */
export const CHUNK_FIND_LIMIT_NON_MEGA = 64
export const CHUNK_FIND_LIMIT_MEGA = 32

/** carveCanyon poses.size 阈值：超过此值视为 mega 峡谷 */
export const MEGA_RAVINE_POSE_THRESHOLD = 200

import { ensureHeap, wasm } from './wasm-bindings'

/**
 * 调用 cubiomes chunk 范围查找函数（ravines/nether_fossils/fossils 共用模式）
 *
 * 统一封装：chunk 范围计算 → buffer 分配 → WASM 调用 → 结果读取 → buffer 释放。
 * 可视范围超过 sizeLimit 时自动分块查找，避免缩放时结构被整片跳过。
 */
export function callChunkFinder(
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
export function callFinderOnce(
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