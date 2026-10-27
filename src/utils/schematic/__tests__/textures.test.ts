import { describe, expect, it } from 'vitest'
import { resolveTexture, type BlockAtlas } from '../textures'

/** 构造一个只带坐标表的图集替身（不加载真实 PNG） */
function makeAtlas(layout: Record<string, [number, number, number, number]>, itemTextures: Record<string, string> = {}): BlockAtlas {
  return {
    image: {} as HTMLImageElement,
    url: 'atlas.png',
    size: [256, 256],
    layout,
    itemTextures: new Map(Object.entries(itemTextures)),
  }
}

const LAYOUT: Record<string, [number, number, number, number]> = {
  'blocks/stone': [0, 0, 16, 16],
  'blocks/oak_planks': [16, 0, 16, 16],
  'blocks/jungle_planks': [32, 0, 16, 16],
  'blocks/stone_bricks': [48, 0, 16, 16],
  'blocks/torch': [64, 0, 16, 16],
  'blocks/dark_oak_torch': [80, 0, 16, 16],
  'items/oak_sign': [96, 0, 16, 16],
  'items/custom_thing': [112, 0, 16, 16],
}

describe('resolveTexture', () => {
  it('should find the block texture by name', () => {
    const atlas = makeAtlas(LAYOUT)
    expect(resolveTexture(atlas, 'minecraft:stone')).toEqual([0, 0, 16, 16])
  })

  it('should prefer the item table mapping over the guessed key', () => {
    const atlas = makeAtlas(LAYOUT, { stone: 'items/custom_thing' })
    expect(resolveTexture(atlas, 'minecraft:stone')).toEqual([112, 0, 16, 16])
  })

  it('should fall back to the material texture for slabs and stairs', () => {
    const atlas = makeAtlas(LAYOUT)
    expect(resolveTexture(atlas, 'minecraft:jungle_slab')).toEqual([32, 0, 16, 16])
    expect(resolveTexture(atlas, 'minecraft:jungle_stairs')).toEqual([32, 0, 16, 16])
    expect(resolveTexture(atlas, 'minecraft:stone_brick_slab')).toEqual([48, 0, 16, 16])
  })

  it('should map wall signs to their item texture', () => {
    const atlas = makeAtlas(LAYOUT)
    expect(resolveTexture(atlas, 'minecraft:oak_wall_sign')).toEqual([96, 0, 16, 16])
  })

  it('should map wall torches back to the standing torch texture', () => {
    const atlas = makeAtlas(LAYOUT)
    expect(resolveTexture(atlas, 'minecraft:wall_torch')).toEqual([64, 0, 16, 16])
  })

  it('should strip blockstate properties before lookup', () => {
    const atlas = makeAtlas(LAYOUT)
    expect(resolveTexture(atlas, 'minecraft:stone[axis=x]')).toEqual([0, 0, 16, 16])
  })

  it('should return null for unknown blocks, air and a missing atlas', () => {
    const atlas = makeAtlas(LAYOUT)
    expect(resolveTexture(atlas, 'minecraft:legacy_17_13')).toBeNull()
    expect(resolveTexture(atlas, 'minecraft:air')).toBeNull()
    expect(resolveTexture(null, 'minecraft:stone')).toBeNull()
  })

  it('should resolve texture when the block state carries properties', () => {
    const atlas = makeAtlas(LAYOUT)
    // 属性只影响朝向与形状，不能影响取到哪张贴图
    expect(resolveTexture(atlas, 'minecraft:jungle_slab[half=top]')).toEqual(
      resolveTexture(atlas, 'minecraft:jungle_slab'),
    )
    expect(resolveTexture(atlas, 'minecraft:stone[facing=east]')).toEqual([0, 0, 16, 16])
    expect(resolveTexture(atlas, 'minecraft:stone_brick_slab[half=top,waterlogged=true]')).toEqual([
      48, 0, 16, 16,
    ])
  })
})
