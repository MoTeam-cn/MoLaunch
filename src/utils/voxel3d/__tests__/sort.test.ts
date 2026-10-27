import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { SORT_FACE_LIMIT, sortFacesByDepth } from '../sort'
import { meshVolume } from '../mesher'
import type { PackData } from '../types'

/** 造一个面：中心点在给定位置，索引为六个连续编号 */
function faceAt(centers: number[], index: number): number[] {
  centers.push(index, 0, 0)
  return [index * 4, index * 4 + 1, index * 4 + 2, index * 4, index * 4 + 2, index * 4 + 3]
}

describe('sortFacesByDepth', () => {
  it('should draw the farthest face first', () => {
    // 三个面分别位于 x=1、x=5、x=3，相机在原点看向 +x
    const centers: number[] = []
    const idx: number[] = [];
    idx.push(...faceAt(centers, 1))
    idx.push(...faceAt(centers, 5))
    idx.push(...faceAt(centers, 3))
    // faceAt 把中心写成 (index,0,0)，这里修正第三个分量为 0 即可
    const sorted = sortFacesByDepth(new Uint32Array(idx), new Float32Array(centers), [0, 0, 0])
    // 每个面的首索引除以 4 就是原始面号；按距离降序应为 5、3、1
    const order = [];
    for (let i = 0; i < sorted.length; i += 6) order.push(sorted[i] / 4);
    expect(order).toEqual([5, 3, 1])
  });

  it('should keep the index buffer length and every triangle intact', () => {
    const centers: number[] = []
    const idx: number[] = []
    for (let i = 0; i < 4; i++) idx.push(...faceAt(centers, i));
    const sorted = sortFacesByDepth(new Uint32Array(idx), new Float32Array(centers), [0, 0, 0])
    expect(sorted.length).toBe(idx.length)
    // 每组六个索引必须仍是同一个四边形：a,b,c 与 a,c,d
    for (let i = 0; i < sorted.length; i += 6) {
      const a = sorted[i]
      const b = sorted[i + 1]
      const c = sorted[i + 2]
      expect(sorted[i + 3]).toBe(a)
      expect(sorted[i + 4]).toBe(c)
      expect(b).not.toBe(a)
      expect(b).not.toBe(c)
    }
  });

  it('should follow the camera when it moves to the other side', () => {
    const centers: number[] = []
    const idx: number[] = []
    idx.push(...faceAt(centers, 0))
    idx.push(...faceAt(centers, 10))
    const fromOrigin = sortFacesByDepth(new Uint32Array(idx), new Float32Array(centers), [0, 0, 0])
    const fromFar = sortFacesByDepth(new Uint32Array(idx), new Float32Array(centers), [20, 0, 0])
    expect(fromOrigin[0] / 4).toBe(10)
    // 相机移到 x=20 之后，x=0 的面成了最远的
    expect(fromFar[0] / 4).toBe(0)
  });

  it('should leave the buffer untouched when over the face limit', () => {
    const centers = new Float32Array(SORT_FACE_LIMIT * 3 + 3)
    const idx = new Uint32Array(SORT_FACE_LIMIT * 6 + 6)
    centers[0] = 100
    const result = sortFacesByDepth(idx, centers, [0, 0, 0])
    expect(result).toBe(idx)
  });

  it('should order real translucent geometry from a meshed volume', () => {
    // 两块玻璃分别位于 z=0 与 z=4，相机在 z=10 正向看过去
    const dir = resolve(process.cwd(), 'node_modules/@mattzh72/lodestone/assets/default-pack')
    const raw = JSON.parse(readFileSync(resolve(dir, 'assets.json'), 'utf8')) as {
      blockstates: Record<string, unknown>
      models: Record<string, unknown>
      textures: Record<string, [number, number, number, number]>
    }
    const pack: PackData = {
      blockstates: raw.blockstates,
      models: raw.models,
      textures: raw.textures,
      atlasSize: [2048, 1248],
      opaqueBlocks: new Set(['stone']),
      transparentBlocks: new Set(['glass']),
    }
    const size: [number, number, number] = [1, 1, 8]
    const blocks = new Uint16Array(size[0] * size[1] * size[2])
    blocks[0 + 0 * size[0] + 0 * size[0] * size[2]] = 1
    blocks[0 + 4 * size[0] + 0 * size[0] * size[2]] = 1
    const mesh = meshVolume({ blocks, palette: ['minecraft:air', 'minecraft:glass'], size, airIndex: 0 }, pack)
    expect(mesh.translucent.faceCount).toBeGreaterThan(0)

    const camera: [number, number, number] = [0.5, 0.5, 10]
    const sorted = sortFacesByDepth(mesh.translucent.indices, mesh.translucent.centers, camera)
    // 排序后第一个面的中心应当比最后一个面更远
    const first = sorted[0] / 4
    const last = sorted[sorted.length - 6] / 4
    const distanceOf = (face: number) => {
      const c = mesh.translucent.centers
      const dx = c[face * 3] - camera[0]
      const dy = c[face * 3 + 1] - camera[1]
      const dz = c[face * 3 + 2] - camera[2]
      return dx * dx + dy * dy + dz * dz
    }
    expect(distanceOf(first)).toBeGreaterThan(distanceOf(last))
  });

  it('should not reorder a single face', () => {
    const centers = new Float32Array([1, 2, 3])
    const idx = new Uint32Array([0, 1, 2, 0, 2, 3])
    expect(sortFacesByDepth(idx, centers, [0, 0, 0])).toBe(idx)
  });
});
