import { describe, expect, it } from 'vitest'
import {
  blockColor,
  blockIndex,
  countSliceBlocks,
  decodeBlocks,
  extractSlice,
  findAirIndex,
  sliceBounds,
  surfaceLayer,
} from '../decode'

/** 把 u16 数组编码为后端使用的 base64（小端） */
function encodeBlocks(values: number[]): string {
  const bytes = new Uint8Array(values.length * 2)
  values.forEach((value, i) => {
    bytes[i * 2] = value & 0xff
    bytes[i * 2 + 1] = (value >> 8) & 0xff
  })
  let binary = ''
  bytes.forEach((b) => {
    binary += String.fromCharCode(b)
  })
  return btoa(binary)
}

describe('decodeBlocks', () => {
  it('should restore u16 values when base64 decoded', () => {
    const decoded = decodeBlocks(encodeBlocks([0, 1, 300, 65535]))
    expect(Array.from(decoded)).toEqual([0, 1, 300, 65535])
  })

  it('should ignore trailing odd byte', () => {
    const decoded = decodeBlocks(btoa(String.fromCharCode(1, 0, 2)))
    expect(Array.from(decoded)).toEqual([1])
  })
})

describe('sliceBounds', () => {
  it('should map y axis to x/z plane', () => {
    expect(sliceBounds([4, 5, 6], 'y')).toEqual({ layers: 5, width: 4, height: 6 })
  })

  it('should map x axis to z/y plane', () => {
    expect(sliceBounds([4, 5, 6], 'x')).toEqual({ layers: 4, width: 6, height: 5 })
  })

  it('should map z axis to x/y plane', () => {
    expect(sliceBounds([4, 5, 6], 'z')).toEqual({ layers: 6, width: 4, height: 5 })
  })
})

describe('blockIndex', () => {
  it('should order x fastest then z then y', () => {
    const size: [number, number, number] = [4, 5, 6]
    expect(blockIndex(size, 0, 0, 0)).toBe(0)
    expect(blockIndex(size, 1, 0, 0)).toBe(1)
    expect(blockIndex(size, 0, 0, 1)).toBe(4)
    expect(blockIndex(size, 0, 1, 0)).toBe(24)
  })
})

describe('extractSlice', () => {
  const size: [number, number, number] = [2, 2, 2]
  // 索引 = x + z*2 + y*4
  const blocks = new Uint16Array([10, 11, 12, 13, 20, 21, 22, 23])

  it('should pick the requested y layer', () => {
    const slice = extractSlice(blocks, size, 'y', 0)
    expect(slice.width).toBe(2)
    expect(slice.height).toBe(2)
    expect(Array.from(slice.cells)).toEqual([10, 11, 12, 13])
  })

  it('should pick the upper y layer', () => {
    const slice = extractSlice(blocks, size, 'y', 1)
    expect(Array.from(slice.cells)).toEqual([20, 21, 22, 23])
  })

  it('should clamp layer beyond range', () => {
    const slice = extractSlice(blocks, size, 'y', 99)
    expect(Array.from(slice.cells)).toEqual([20, 21, 22, 23])
  })

  it('should slice along x axis with z as width', () => {
    const slice = extractSlice(blocks, size, 'x', 0)
    expect(slice.width).toBe(2)
    expect(slice.height).toBe(2)
    // u = z, v = y → idx(0, v, u)
    expect(Array.from(slice.cells)).toEqual([10, 12, 20, 22])
  })
})

describe('blockColor', () => {
  it('should use known color table for common blocks', () => {
    expect(blockColor('minecraft:stone')).toBe('#7d7d7d')
  })

  it('should be stable for unknown blocks', () => {
    const first = blockColor('minecraft:some_unknown_block')
    const second = blockColor('minecraft:some_unknown_block')
    expect(first).toBe(second)
    expect(first.startsWith('hsl(')).toBe(true)
  })
})

describe('palette helpers', () => {
  it('should locate air index', () => {
    expect(findAirIndex(['minecraft:stone', 'minecraft:air'])).toBe(1)
    expect(findAirIndex(['minecraft:stone'])).toBe(-1)
  })

  it('should count non air cells only', () => {
    const cells = new Uint16Array([0, 1, 0, 1])
    expect(countSliceBlocks(cells, ['minecraft:air', 'minecraft:stone'], 0)).toBe(2)
  })
})

describe('surfaceLayer', () => {
  it('should land on the middle of a sloped terrain when heights vary', () => {
    // 山坡从 y=40 爬到 y=90（topHeight 存的是 y+1），中位数落在 y=65
    const heights = [41, 61, 71, 91, 51];
    expect(surfaceLayer(heights)).toBe(60);
  });

  it('should land on the roof when the build is floating with a flat top', () => {
    // 悬空建筑：所有列都止于 y=80
    expect(surfaceLayer([81, 81, 81, 81])).toBe(80);
  });

  it('should return -1 when every column is empty', () => {
    expect(surfaceLayer([0, 0, 0])).toBe(-1);
  });

  it('should never return a negative layer for a single-block build', () => {
    expect(surfaceLayer([1, 0, 0])).toBe(0);
  });
});
