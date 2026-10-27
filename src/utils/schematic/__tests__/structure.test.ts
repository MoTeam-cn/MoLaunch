import { describe, expect, it } from 'vitest'
import type { Resources } from '@mattzh72/lodestone'
import { buildStructure } from '../structure'

/** 只提供 getBlockFlags 的替身：stone / dirt 不透明，water 是液体且不透明标记为假 */
function fakeResources(): Resources {
  const opaque = new Set(['stone', 'dirt'])
  return {
    getBlockFlags: (id: { path: string }) => ({ opaque: opaque.has(id.path) }),
  } as unknown as Resources
}

function volumeOf(
  size: [number, number, number],
  fill: (x: number, y: number, z: number) => number,
) {
  const [sx, sy, sz] = size
  const blocks = new Uint16Array(sx * sy * sz)
  for (let y = 0; y < sy; y++) {
    for (let z = 0; z < sz; z++) {
      for (let x = 0; x < sx; x++) {
        blocks[x + z * sx + y * sx * sz] = fill(x, y, z)
      }
    }
  }
  return { blocks, palette: ['minecraft:air', 'minecraft:stone', 'minecraft:water'], size, airIndex: 0 }
}

describe('buildStructure', () => {

  it('should keep every non-air block by default', () => {
    // 默认不剔除：渲染器不知道被剔掉的位置原本有方块，会把洞当成空气画出内侧的面
    const built = buildStructure(volumeOf([5, 5, 5], () => 1), fakeResources())
    expect(built.total).toBe(125)
    expect(built.kept).toBe(125)
  });

  it('should drop fully enclosed blocks when culling is on', () => {
    const built = buildStructure(volumeOf([5, 5, 5], () => 1), fakeResources(), {
      cullOccluded: true,
    })
    expect(built.total).toBe(125)
    // 5×5×5 内部 3×3×3 = 27 个被完全包围
    expect(built.kept).toBe(125 - 27)
  });

  it('should keep every block of a single layer even with culling on', () => {
    const built = buildStructure(volumeOf([4, 1, 4], () => 1), fakeResources(), {
      cullOccluded: true,
    })
    expect(built.kept).toBe(16)
  });

  it('should keep cavity walls when a hollow room is carved out', () => {
    const hollow = volumeOf([5, 5, 5], (x, y, z) => (x === 2 && y === 2 && z === 2 ? 0 : 1))
    const built = buildStructure(hollow, fakeResources(), { cullOccluded: true })
    expect(built.kept).toBe(125 - 27 + 6)
  });

  it('should not treat liquid neighbours as opaque', () => {
    const built = buildStructure(volumeOf([3, 3, 3], (_x, _y, z) => (z < 2 ? 1 : 2)), fakeResources(), {
      cullOccluded: true,
    })
    expect(built.kept).toBeGreaterThan(0)
  });
});
