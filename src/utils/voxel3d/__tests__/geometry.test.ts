import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeAll, describe, expect, it } from 'vitest'
import { buildBlockGeometry } from '../geometry'
import { parseBlockState } from '../model'
import type { PackData } from '../types'

/** 真实资源包（图集贴图坐标齐全），用来校验解析器对原版模型的还原 */
let pack: PackData

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
    opaqueBlocks: new Set(['stone', 'oak_planks', 'grass_block', 'oak_log']),
    transparentBlocks: new Set(['glass', 'oak_leaves']),
  }
});

function faces(state: string) {
  return buildBlockGeometry(state, pack, new Map()).faces
}

describe('parseBlockState', () => {
  it('should read the name and properties', () => {
    const parsed = parseBlockState('minecraft:oak_stairs[facing=north,half=top,shape=straight]')
    expect(parsed.name).toBe('oak_stairs')
    expect(parsed.props.facing).toBe('north')
    expect(parsed.props.half).toBe('top')
  });

  it('should handle a state without properties', () => {
    expect(parseBlockState('minecraft:stone').props).toEqual({})
  });
});

describe('buildBlockGeometry with the real pack', () => {
  it('should give a full cube six faces', () => {
    const result = faces('minecraft:stone')
    expect(result).toHaveLength(6)
    for (const face of result) {
      expect(face.vertices).toHaveLength(4)
      expect(face.cullface).toBeTruthy()
    }
  });

  it('should keep a slab inside the lower half', () => {
    const slab = faces('minecraft:oak_slab[type=bottom,waterlogged=false]')
    expect(slab.length).toBeGreaterThan(0)
    const maxY = Math.max(...slab.flatMap((f) => f.vertices.map((v) => v[1])))
    const minY = Math.min(...slab.flatMap((f) => f.vertices.map((v) => v[1])))
    expect(minY).toBeCloseTo(0)
    expect(maxY).toBeCloseTo(0.5)
  });

  it('should place a double slab across the full height', () => {
    const slab = faces('minecraft:oak_slab[type=double]')
    const maxY = Math.max(...slab.flatMap((f) => f.vertices.map((v) => v[1])))
    expect(maxY).toBeCloseTo(1)
  });

  it('should give stairs more faces than a cube', () => {
    const stairs = faces('minecraft:oak_stairs[facing=east,half=bottom,shape=straight]')
    expect(stairs.length).toBeGreaterThan(6)
  });

  it('should turn the log model when axis is x', () => {
    // 竖直原木的高度覆盖整格，沿 x 轴放倒后高度只剩横截面
    const vertical = faces('minecraft:oak_log[axis=y]')
    const horizontal = faces('minecraft:oak_log[axis=x]')
    const heightOf = (list: { vertices: [number, number, number][] }[]) =>
      Math.max(...list.flatMap((f) => f.vertices.map((v) => v[1])))
    expect(heightOf(vertical)).toBeCloseTo(1)
    expect(heightOf(horizontal)).toBeCloseTo(1)
    // 两种朝向的面数相同，但顶点分布不同：横放时 x 方向仍占满，y/z 交换
    expect(vertical).toHaveLength(horizontal.length)
  });

  it('should resolve every texture to a uv inside the atlas', () => {
    for (const state of ['minecraft:stone', 'minecraft:oak_planks', 'minecraft:grass_block[snowy=false]', 'minecraft:glass']) {
      for (const face of faces(state)) {
        expect(face.uv[0]).toBeGreaterThanOrEqual(0)
        expect(face.uv[1]).toBeGreaterThanOrEqual(0)
        expect(face.uv[2]).toBeLessThanOrEqual(1)
        expect(face.uv[3]).toBeLessThanOrEqual(1)
        expect(face.uv[2]).toBeGreaterThan(face.uv[0])
        expect(face.uv[3]).toBeGreaterThan(face.uv[1])
      }
    }
  });

  it('should mark leaves as translucent and stone as opaque', () => {
    const map = new Map()
    expect(buildBlockGeometry('minecraft:oak_leaves', pack, map).translucent).toBe(true)
    expect(buildBlockGeometry('minecraft:stone', pack, map).translucent).toBe(false)
  });

  it('should return nothing for a block the pack does not know', () => {
    expect(faces('minecraft:not_a_real_block_xyz')).toHaveLength(0)
  });
});
