import { describe, expect, it } from 'vitest'
import { meshVolume } from '../mesher'
import type { PackData, VoxelVolume } from '../types'

function cubeElement() {
  const faces: Record<string, unknown> = {}
  for (const dir of ['down', 'up', 'north', 'south', 'west', 'east']) {
    faces[dir] = { texture: '#all', cullface: dir, uv: [0, 0, 16, 16] }
  }
  return { from: [0, 0, 0], to: [16, 16, 16], faces }
}

function testPack(): PackData {
  return {
    blockstates: {
      stone: { variants: { '': { model: 'block/cube_all' } } },
      glass: { variants: { '': { model: 'block/glass_model' } } },
    },
    models: {
      'block/cube_all': { textures: { all: 'block/stone' }, elements: [cubeElement()] },
      'block/glass_model': { textures: { all: 'block/stone' }, elements: [cubeElement()] },
    },
    textures: { 'block/stone': [0, 0, 16, 16] },
    atlasSize: [256, 256],
    opaqueBlocks: new Set(['stone']),
    transparentBlocks: new Set(['glass']),
  }
}

function volumeOf(
  size: [number, number, number],
  fill: (x: number, y: number, z: number) => number,
  palette: string[] = ['minecraft:air', 'minecraft:stone'],
): VoxelVolume {
  const [sx, sy, sz] = size
  const blocks = new Uint16Array(sx * sy * sz)
  for (let y = 0; y < sy; y++) {
    for (let z = 0; z < sz; z++) {
      for (let x = 0; x < sx; x++) {
        blocks[x + z * sx + y * sx * sz] = fill(x, y, z)
      }
    }
  }
  return { blocks, palette, size, airIndex: 0 }
}

describe('meshVolume', () => {
  it('should emit six faces for a lone block', () => {
    const data = meshVolume(volumeOf([1, 1, 1], () => 1), testPack())
    expect(data.opaque.faceCount).toBe(6)
    expect(data.opaque.positions.length).toBe(6 * 12)
    expect(data.opaque.indices.length).toBe(6 * 6)
  });

  it('should split the shared face between two touching blocks', () => {
    const data = meshVolume(volumeOf([2, 1, 1], () => 1), testPack())
    expect(data.opaque.faceCount).toBe(10)
  });

  it('should keep only the shell of a solid slab', () => {
    const data = meshVolume(volumeOf([3, 3, 3], () => 1), testPack())
    // 3×3×3 的可见面：六个方向各 9 面
    expect(data.opaque.faceCount).toBe(3 * 3 * 6)
  });

  it('should emit nothing for air', () => {
    const data = meshVolume(volumeOf([2, 2, 2], () => 0), testPack())
    expect(data.opaque.faceCount).toBe(0)
    expect(data.opaque.positions.length).toBe(0)
  });

  it('should not treat transparent blocks as occluders', () => {
    const data = meshVolume(
      volumeOf([2, 1, 1], (x) => (x === 0 ? 1 : 2), ['minecraft:air', 'minecraft:stone', 'minecraft:glass']),
      testPack(),
    )
    // 石头 6 面走不透明，玻璃 6 面走半透明；玻璃朝石头那一面被石头挡住 → 合计 11
    expect(data.opaque.faceCount + data.translucent.faceCount).toBe(11)
    expect(data.translucent.faceCount).toBe(5)
  });

  it('should keep every axis inside the voxel bounds', () => {
    const data = meshVolume(volumeOf([2, 1, 1], (x) => (x === 0 ? 1 : 0)), testPack())
    for (let i = 0; i < data.opaque.positions.length; i += 3) {
      expect(data.opaque.positions[i]).toBeGreaterThanOrEqual(0)
      expect(data.opaque.positions[i]).toBeLessThanOrEqual(1.0001)
      expect(data.opaque.positions[i + 2]).toBeGreaterThanOrEqual(0)
      expect(data.opaque.positions[i + 2]).toBeLessThanOrEqual(1.0001)
    }
  });

  it('should give every face four uv pairs inside the atlas', () => {
    const data = meshVolume(volumeOf([1, 1, 1], () => 1), testPack())
    expect(data.opaque.uvs.length).toBe(6 * 8)
    for (const value of data.opaque.uvs) {
      expect(value).toBeGreaterThanOrEqual(0)
      expect(value).toBeLessThanOrEqual(1)
    }
  });
});
