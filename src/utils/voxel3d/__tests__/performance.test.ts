import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeAll, describe, expect, it } from 'vitest'
import { createMesherCache, meshVolume } from '../mesher'
import type { PackData, VoxelVolume } from '../types'

let pack: PackData;

beforeAll(() => {
  const path = resolve(process.cwd(), 'node_modules/@mattzh72/lodestone/assets/default-pack/assets.json')
  const raw = JSON.parse(readFileSync(path, 'utf8')) as {
    blockstates: Record<string, unknown>
    models: Record<string, unknown>
    textures: Record<string, [number, number, number, number]>
  }
  pack = {
    blockstates: raw.blockstates,
    models: raw.models,
    textures: raw.textures,
    atlasSize: [2048, 1248],
    opaqueBlocks: new Set(['stone', 'oak_planks', 'grass_block']),
    transparentBlocks: new Set(['glass', 'oak_leaves']),
  }
});

/** 造一片起伏地形：底部实心，顶上叠一层草 */
function terrain(size: [number, number, number], height: number): VoxelVolume {
  const [sx, sy, sz] = size
  const blocks = new Uint16Array(sx * sy * sz)
  let filled = 0
  for (let z = 0; z < sz; z++) {
    for (let x = 0; x < sx; x++) {
      const h = Math.floor(height / 2 + Math.sin(x / 9) * 6 + Math.cos(z / 11) * 5)
      for (let y = 0; y < Math.min(h, sy); y++) {
        blocks[x + z * sx + y * sx * sz] = y === h - 1 ? 2 : 1
        filled++
      }
    }
  }
  return { blocks, palette: ['minecraft:air', 'minecraft:stone', 'minecraft:grass_block'], size, airIndex: 0 }
}

describe('meshVolume at projection scale', () => {
  it('should mesh a 147×256×147 volume within a few seconds', () => {
    const volume = terrain([147, 256, 147], 40)
    const cache = createMesherCache()
    const started = performance.now()
    const mesh = meshVolume(volume, pack, cache)
    const elapsed = performance.now() - started

    // 只有表面该出面：远小于方块数 × 6
    expect(mesh.faceCount).toBeGreaterThan(0)
    expect(mesh.faceCount).toBeLessThan(1_000_000)
    expect(elapsed).toBeLessThan(8000)
    console.log(`[112] 面数 ${mesh.faceCount.toLocaleString()}，耗时 ${elapsed.toFixed(0)} ms`);
  });

  it('should reuse cached block geometry on the second pass', () => {
    const volume = terrain([40, 40, 40], 20)
    const cache = createMesherCache()
    meshVolume(volume, pack, cache)
    const started = performance.now()
    meshVolume(volume, pack, cache)
    const elapsed = performance.now() - started
    expect(cache.geometry.size).toBeGreaterThan(0)
    expect(elapsed).toBeLessThan(3000)
  });
});
