/**
 * 半透明面的视角排序
 *
 * 半透明网格不写深度，混合顺序直接决定观感：必须把离相机远的面先画。
 * 这里只重排索引，顶点、UV、着色原样复用，代价远小于重建几何。
 */
import type { Vec3 } from './orbit'

/** 超过这个面数就不排了：排序是 O(n log n)，面太多会让拖拽掉帧 */
export const SORT_FACE_LIMIT = 50000

/** 相机移动超过这个距离才值得重排（单位：格） */
export const SORT_CAMERA_EPSILON = 0.5

/**
 * 按面中心到相机的距离降序重排索引。
 *
 * 面数少于 2 或超过上限时原样返回，调用方不必特判。
 */
export function sortFacesByDepth(
  indices: Uint32Array,
  centers: Float32Array,
  camera: Vec3,
  limit = SORT_FACE_LIMIT,
): Uint32Array {
  const faceCount = Math.floor(centers.length / 3)
  if (faceCount < 2 || faceCount > limit) return indices

  const distance = new Float32Array(faceCount)
  for (let f = 0; f < faceCount; f++) {
    const dx = centers[f * 3] - camera[0]
    const dy = centers[f * 3 + 1] - camera[1]
    const dz = centers[f * 3 + 2] - camera[2]
    distance[f] = dx * dx + dy * dy + dz * dz
  }

  const order = new Array<number>(faceCount)
  for (let f = 0; f < faceCount; f++) order[f] = f
  // 远的先画：距离大的排前面
  order.sort((a, b) => distance[b] - distance[a])

  const out = new Uint32Array(indices.length)
  for (let i = 0; i < faceCount; i++) {
    const src = order[i] * 6
    const dst = i * 6
    out[dst] = indices[src]
    out[dst + 1] = indices[src + 1]
    out[dst + 2] = indices[src + 2]
    out[dst + 3] = indices[src + 3]
    out[dst + 4] = indices[src + 4]
    out[dst + 5] = indices[src + 5]
  }
  return out
}
