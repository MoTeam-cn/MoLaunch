/**
 * 合成配方生成器核心逻辑单测
 */
import { describe, expect, it } from 'vitest'
import {
  itemRefToString,
  parseIdentifier,
  stripData,
} from '../identifier'
import {
  coerceRecipeTypeForVersion,
  getSupportedRecipeTypes,
  getJavaVersionMeta,
  isRecipeTypeAvailable,
  isVersionAtLeast,
  supportsRecipeCategory,
  supportsShowNotification,
} from '../versions'
import { formatRecipeJson } from '../formatter'
import { validateRecipe } from '../validation'
import { createDatapackFiles, createPackMcmeta, sanitizeRecipeName } from '../datapack'
import { generateRecipeJson, getCraftingGridValues } from '../recipe-engine'
import { loadVersionItems, loadVersionTags } from '../resources'
import { tagLabel } from '../tag-zh'
import { resolveTagDisplay } from '../tag-resolve'
import type { RecipeSlotContext, RecipeState } from '../types'

function makeContext(): RecipeSlotContext {
  return {
    itemsById: {},
    customItemsByUid: {},
    customTagsByUid: {},
    vanillaTags: { 'minecraft:planks': ['minecraft:oak_planks'] },
  }
}

function makeState(overrides?: Partial<RecipeState>): RecipeState {
  return {
    id: 'test',
    recipeType: 'crafting',
    group: '',
    category: 'misc',
    showNotification: true,
    nameMode: 'manual',
    name: 'test',
    slots: {},
    crafting: { shapeless: false, keepWhitespace: false, twoByTwo: false },
    cooking: { time: null, experience: 0 },
    smithing: { trimPattern: '' },
    ...overrides,
  }
}

describe('identifier', () => {
  it('parses namespaced and bare identifiers', () => {
    expect(parseIdentifier('minecraft:stone')).toEqual({ namespace: 'minecraft', id: 'stone' })
    expect(parseIdentifier('stone')).toEqual({ namespace: 'minecraft', id: 'stone' })
    expect(parseIdentifier('minecraft:deepslate_bricks')).toEqual({
      namespace: 'minecraft',
      id: 'deepslate_bricks',
    })
  })

  it('parses legacy data suffix (1.12 style)', () => {
    expect(parseIdentifier('minecraft:stone:1')).toEqual({
      namespace: 'minecraft',
      id: 'stone',
      data: 1,
    })
  })

  it('rejects invalid identifiers', () => {
    expect(parseIdentifier('')).toBeNull()
    expect(parseIdentifier('Invalid:Item')).toBeNull()
    expect(parseIdentifier('minecraft:stone:abc')).toBeNull()
  })

  it('round-trips via itemRefToString and stripData', () => {
    expect(itemRefToString({ namespace: 'minecraft', id: 'stone', data: 3 })).toBe(
      'minecraft:stone:3',
    )
    expect(stripData('minecraft:stone:2')).toBe('minecraft:stone')
    expect(stripData('minecraft:stone')).toBe('minecraft:stone')
  })
})

describe('versions', () => {
  it('compares minecraft version strings', () => {
    expect(isVersionAtLeast('1.21', '1.20')).toBe(true)
    expect(isVersionAtLeast('1.20', '1.21')).toBe(false)
    expect(isVersionAtLeast('1.21.2', '1.21.11')).toBe(false)
    expect(isVersionAtLeast('26.1', '26.2')).toBe(false)
    expect(isVersionAtLeast('1.21.11', '1.21.11')).toBe(true)
  })

  it('gates recipe type availability per version', () => {
    expect(isRecipeTypeAvailable('1.12', 'crafting')).toBe(true)
    expect(isRecipeTypeAvailable('1.12', 'smelting')).toBe(false)
    expect(isRecipeTypeAvailable('1.13', 'smelting')).toBe(true)
    expect(isRecipeTypeAvailable('1.18', 'smithing')).toBe(true)
    expect(isRecipeTypeAvailable('1.19', 'smithing')).toBe(false)
    expect(isRecipeTypeAvailable('1.19', 'smithing_trim')).toBe(true)
  })

  it('lists supported types and coerces unsupported ones', () => {
    expect(getSupportedRecipeTypes('1.12')).toEqual(['crafting'])
    expect(getSupportedRecipeTypes('1.19')).toContain('smithing_trim')
    expect(coerceRecipeTypeForVersion('smithing', '1.19')).toBe('crafting')
    expect(coerceRecipeTypeForVersion('crafting', '1.12')).toBe('crafting')
  })

  it('reports pack metadata and category capability', () => {
    expect(getJavaVersionMeta('1.12').packFormat).toBeNull()
    expect(getJavaVersionMeta('1.21').recipeDir).toBe('recipe')
    expect(getJavaVersionMeta('1.20').recipeDir).toBe('recipes')
    expect(supportsRecipeCategory('1.19', 'crafting')).toBe(true)
    expect(supportsRecipeCategory('1.18', 'crafting')).toBe(false)
  })

  it('gates show_notification support', () => {
    expect(supportsShowNotification('1.19', 'crafting', false)).toBe(true)
    expect(supportsShowNotification('1.18', 'crafting', false)).toBe(false)
    expect(supportsShowNotification('1.20', 'crafting', true)).toBe(false)
    expect(supportsShowNotification('26.1', 'crafting', true)).toBe(true)
  })
})

describe('formatter', () => {
  it('formats shaped recipe with object ingredient (1.20)', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'item', id: 'minecraft:iron_ingot' },
        'crafting.result': { kind: 'item', id: 'minecraft:iron_block', count: 2 },
      },
    })
    const json = formatRecipeJson(state, '1.20', makeContext())
    expect(json.type).toBe('minecraft:crafting_shaped')
    expect(json.pattern).toEqual(['A'])
    expect(json.key).toEqual({ A: { item: 'minecraft:iron_ingot' } })
    expect(json.result).toEqual({ id: 'minecraft:iron_block', count: 2 })
  })

  it('uses string ingredient format on 1.21.2+', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'item', id: 'minecraft:iron_ingot' },
        'crafting.result': { kind: 'item', id: 'minecraft:iron_block' },
      },
    })
    const json = formatRecipeJson(state, '1.21.2', makeContext())
    expect(json.key).toEqual({ A: 'minecraft:iron_ingot' })
    expect(json.result).toEqual({ id: 'minecraft:iron_block' })
  })

  it('emits ingredient count on 1.21.2+ and omits it on older versions', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'item', id: 'minecraft:oak_planks', count: 2 },
        'crafting.2': { kind: 'item', id: 'minecraft:oak_sapling' },
        'crafting.result': { kind: 'item', id: 'minecraft:stick' },
      },
    })
    expect(formatRecipeJson(state, '1.21.2', makeContext()).key).toEqual({
      A: { item: 'minecraft:oak_planks', count: 2 },
      B: 'minecraft:oak_sapling',
    })
    expect(formatRecipeJson(state, '1.20', makeContext()).key).toEqual({
      A: { item: 'minecraft:oak_planks' },
      B: { item: 'minecraft:oak_sapling' },
    })
  })

  it('emits cooking ingredient count on 1.21.2+', () => {
    const state = makeState({
      recipeType: 'smelting',
      slots: {
        'cooking.ingredient': { kind: 'item', id: 'minecraft:iron_ore', count: 3 },
        'cooking.result': { kind: 'item', id: 'minecraft:iron_ingot' },
      },
    })
    expect(formatRecipeJson(state, '1.21.2', makeContext()).ingredient).toEqual({
      item: 'minecraft:iron_ore',
      count: 3,
    })
  })

  it('formats legacy 1.12 recipe with separate data field and bare type', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'item', id: 'minecraft:wool:1' },
        'crafting.result': { kind: 'item', id: 'minecraft:wool:2' },
      },
    })
    const json = formatRecipeJson(state, '1.12', makeContext())
    expect(json.type).toBe('crafting_shaped')
    expect(json.key).toEqual({ A: { item: 'minecraft:wool', data: 1 } })
    expect(json.result).toEqual({ item: 'minecraft:wool', data: 2 })
  })

  it('formats shapeless ingredients list', () => {
    const state = makeState({
      recipeType: 'crafting',
      crafting: { shapeless: true, keepWhitespace: false, twoByTwo: false },
      slots: {
        'crafting.1': { kind: 'item', id: 'minecraft:oak_planks' },
        'crafting.2': { kind: 'vanilla_tag', id: 'minecraft:planks' },
        'crafting.result': { kind: 'item', id: 'minecraft:stick' },
      },
    })
    const json = formatRecipeJson(state, '1.20', makeContext())
    expect(json.type).toBe('minecraft:crafting_shapeless')
    expect(json.ingredients).toEqual([
      { item: 'minecraft:oak_planks' },
      { tag: 'minecraft:planks' },
    ])
  })

  it('emits string tag references on 1.21.2+', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'vanilla_tag', id: 'minecraft:planks' },
        'crafting.result': { kind: 'item', id: 'minecraft:oak_planks' },
      },
    })
    const json = formatRecipeJson(state, '1.21.2', makeContext())
    expect(json.key).toEqual({ A: '#minecraft:planks' })
  })

  it('uses string result for cooking before 1.20 and object after', () => {
    const oldState = makeState({
      recipeType: 'smelting',
      slots: {
        'cooking.ingredient': { kind: 'item', id: 'minecraft:iron_ore' },
        'cooking.result': { kind: 'item', id: 'minecraft:iron_ingot' },
      },
    })
    expect(formatRecipeJson(oldState, '1.14', makeContext()).result).toBe('minecraft:iron_ingot')

    const newState = makeState({
      recipeType: 'smelting',
      slots: {
        'cooking.ingredient': { kind: 'item', id: 'minecraft:iron_ore' },
        'cooking.result': { kind: 'item', id: 'minecraft:iron_ingot' },
      },
    })
    expect(formatRecipeJson(newState, '1.20', makeContext()).result).toEqual({
      id: 'minecraft:iron_ingot',
    })
  })

  it('nests stonecutter result and keeps count at top level before 1.20', () => {
    const state = makeState({
      recipeType: 'stonecutter',
      slots: {
        'stonecutter.ingredient': { kind: 'item', id: 'minecraft:stone' },
        'stonecutter.result': { kind: 'item', id: 'minecraft:stone_slab', count: 2 },
      },
    })
    const json = formatRecipeJson(state, '1.19', makeContext())
    expect(json.result).toEqual({ result: 'minecraft:stone_slab', count: 2 })
  })

  it('writes show_notification false only when supported and disabled', () => {
    const state = makeState({
      showNotification: false,
      slots: { 'crafting.result': { kind: 'item', id: 'minecraft:stone' } },
    })
    const json = formatRecipeJson(state, '1.20', makeContext())
    expect(json.show_notification).toBe(false)
  })

  it('omits smithing result for smithing_trim and writes pattern on 1.21.5+', () => {
    const state = makeState({
      recipeType: 'smithing_trim',
      smithing: { trimPattern: 'minecraft:silence_armor_trim_smithing_template' },
      slots: {
        'smithing.template': { kind: 'item', id: 'minecraft:smithing_template' },
        'smithing.base': { kind: 'item', id: 'minecraft:iron_chestplate' },
        'smithing.addition': { kind: 'item', id: 'minecraft:diamond' },
      },
    })
    const json = formatRecipeJson(state, '1.21.5', makeContext())
    expect(json.result).toBeUndefined()
    expect(json.pattern).toBe('minecraft:silence_armor_trim_smithing_template')
  })

  it('uses bare type names on 1.13 and namespaced from 1.14', () => {
    const state = makeState({ slots: { 'crafting.result': { kind: 'item', id: 'minecraft:stone' } } })
    expect(formatRecipeJson(state, '1.13', makeContext()).type).toBe('crafting_shaped')
    expect(formatRecipeJson(state, '1.14', makeContext()).type).toBe('minecraft:crafting_shaped')
  })
})

describe('validation', () => {
  it('flags missing ingredient and result', () => {
    const codes = validateRecipe(makeState(), '1.20', makeContext()).map((issue) => issue.code)
    expect(codes).toContain('missing-ingredient')
    expect(codes).toContain('missing-result')
  })

  it('rejects tags in result slots', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'item', id: 'minecraft:oak_planks' },
        'crafting.result': { kind: 'vanilla_tag', id: 'minecraft:planks' },
      },
    })
    const codes = validateRecipe(state, '1.20', makeContext()).map((issue) => issue.code)
    expect(codes).toContain('tag-in-result')
  })

  it('rejects tags on 1.12', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'vanilla_tag', id: 'minecraft:planks' },
        'crafting.result': { kind: 'item', id: 'minecraft:oak_planks' },
      },
    })
    const codes = validateRecipe(state, '1.12', makeContext()).map((issue) => issue.code)
    expect(codes).toContain('tags-not-supported')
  })

  it('flags missing smithing_trim pattern on supported versions', () => {
    const state = makeState({
      recipeType: 'smithing_trim',
      slots: {
        'smithing.template': { kind: 'item', id: 'minecraft:smithing_template' },
        'smithing.base': { kind: 'item', id: 'minecraft:iron_chestplate' },
        'smithing.addition': { kind: 'item', id: 'minecraft:diamond' },
      },
    })
    const codes = validateRecipe(state, '1.21.5', makeContext()).map((issue) => issue.code)
    expect(codes).toContain('missing-trim-pattern')
  })

  it('passes a complete valid recipe', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'item', id: 'minecraft:oak_planks' },
        'crafting.2': { kind: 'item', id: 'minecraft:oak_planks' },
        'crafting.result': { kind: 'item', id: 'minecraft:stick', count: 4 },
      },
    })
    expect(validateRecipe(state, '1.20', makeContext())).toHaveLength(0)
  })
})

describe('datapack', () => {
  it('sanitizes recipe names', () => {
    expect(sanitizeRecipeName('My Recipe!', 'x')).toBe('my_recipe')
    expect(sanitizeRecipeName('  stone_bricks   ', 'x')).toBe('stone_bricks')
    expect(sanitizeRecipeName('!!!', 'x')).toBe('x')
  })

  it('writes pack_format number or min/max range', () => {
    expect(createPackMcmeta(48)).toContain('"pack_format": 48')
    expect(createPackMcmeta([88, 0])).toContain('"min_format": 88')
    expect(createPackMcmeta([88, 0])).toContain('"max_format": 0')
  })

  it('places recipes under recipes/ before 1.21 and recipe/ after', () => {
    const recipe = { name: 'test', json: { type: 'minecraft:crafting_shaped' } }
    const paths = createDatapackFiles('1.20', [recipe], []).map((file) => file.path)
    expect(paths).toContain('data/crafting/recipes/test.json')
    expect(paths).toContain('pack.mcmeta')

    const paths21 = createDatapackFiles('1.21', [recipe], []).map((file) => file.path)
    expect(paths21).toContain('data/crafting/recipe/test.json')
  })

  it('writes custom tag files under the tag namespace', () => {
    const recipe = { name: 'test', json: {} }
    const tag = { namespace: 'mymod', id: 'wheels', values: ['minecraft:iron_ingot'] }
    const file = createDatapackFiles('1.21', [recipe], [tag]).find((entry) =>
      entry.path.includes('wheels'),
    )
    expect(file?.path).toBe('data/mymod/tags/item/wheels.json')
    expect(file?.content).toContain('minecraft:iron_ingot')
  })

  it('throws for versions without datapack support', () => {
    expect(() => createDatapackFiles('1.12', [], [])).toThrow(/不支持数据包/)
  })
})

describe('recipe-engine', () => {
  it('disables 2x2 corner cells via grid values', () => {
    const state = makeState({ crafting: { shapeless: false, keepWhitespace: false, twoByTwo: true } })
    const values = getCraftingGridValues(state)
    expect(values).toHaveLength(9)
    expect(values[2]).toBeUndefined()
    expect(values[5]).toBeUndefined()
    expect(values[8]).toBeUndefined()
    expect(values[0]).toBeUndefined()
  })

  it('generates a valid recipe JSON', () => {
    const state = makeState({
      slots: {
        'crafting.1': { kind: 'item', id: 'minecraft:iron_ingot' },
        'crafting.result': { kind: 'item', id: 'minecraft:iron_block' },
      },
    })
    const json = generateRecipeJson(state, '1.20', makeContext())
    expect(json.type).toBe('minecraft:crafting_shaped')
    expect(json.result).toEqual({ id: 'minecraft:iron_block' })
  })

  it('throws a Chinese message for invalid recipes', () => {
    expect(() => generateRecipeJson(makeState(), '1.20', makeContext())).toThrow(/缺少/)
  })
})

describe('resources', () => {
  it('loads version items with a non-empty item list', async () => {
    const items = await loadVersionItems('1.21')
    expect(items.length).toBeGreaterThan(0)
    expect(items.some((item) => item.texture)).toBe(true)
  })

  it('loads version tags as a map of many vanilla tags', async () => {
    const tags = await loadVersionTags('1.21')
    const keys = Object.keys(tags)
    expect(keys.length).toBeGreaterThan(10)
    expect(keys).toContain('minecraft:planks')
    expect(tags['minecraft:planks'].length).toBeGreaterThan(0)
  })
})

describe('tag-zh', () => {
  it('labels every shipped vanilla tag with a Chinese name', async () => {
    const tags = await loadVersionTags('26.2')
    for (const id of Object.keys(tags)) {
      const label = tagLabel(id)
      expect(label).not.toBe('')
      expect(label).not.toBe(id)
    }
  })

  it('falls back to a readable name for unknown tags', () => {
    expect(tagLabel('minecraft:foo_bar/baz')).toBe('Foo Bar / Baz')
  })
})

describe('tag-resolve', () => {
  function makeTagContext(): RecipeSlotContext {
    return {
      itemsById: {
        'minecraft:oak_planks': {
          id: 'minecraft:oak_planks',
          name: 'Oak Planks',
          zh: '橡木木板',
          texture: 'oak_planks',
        },
        'minecraft:spruce_planks': {
          id: 'minecraft:spruce_planks',
          name: 'Spruce Planks',
          zh: '云杉木板',
          texture: 'spruce_planks',
        },
        'minecraft:air': { id: 'minecraft:air', name: 'Air', zh: '', texture: null },
      },
      customItemsByUid: {},
      customTagsByUid: {
        'tag-1': {
          uid: 'tag-1',
          id: 'mymod:wood',
          values: [
            { type: 'item', id: 'minecraft:oak_planks' },
            { type: 'tag', id: 'minecraft:planks' },
            { type: 'item', id: 'minecraft:air' },
          ],
        },
      },
      vanillaTags: {
        'minecraft:planks': ['minecraft:oak_planks', 'minecraft:spruce_planks', 'minecraft:air'],
      },
    }
  }

  it('resolves vanilla tag display with the first textured member', () => {
    const display = resolveTagDisplay({ kind: 'vanilla_tag', id: 'minecraft:planks' }, makeTagContext())
    expect(display.label).toBe(`#${tagLabel('minecraft:planks')}`)
    expect(display.texture).toBe('oak_planks')
    expect(display.members.map((member) => member.id)).toEqual([
      'minecraft:oak_planks',
      'minecraft:spruce_planks',
    ])
  })

  it('expands custom tag members, including nested tag references, deduped', () => {
    const display = resolveTagDisplay({ kind: 'custom_tag', uid: 'tag-1' }, makeTagContext())
    expect(display.label).toBe('#mymod:wood')
    expect(display.texture).toBe('oak_planks')
    const ids = display.members.map((member) => member.id)
    expect(ids).toContain('minecraft:oak_planks')
    expect(ids).toContain('minecraft:spruce_planks')
    expect(ids).not.toContain('minecraft:air')
    expect(ids).toHaveLength(2)
  })

  it('falls back to null texture when no member has a texture', () => {
    const context = makeTagContext()
    context.vanillaTags['minecraft:empty'] = ['minecraft:air']
    const display = resolveTagDisplay({ kind: 'vanilla_tag', id: 'minecraft:empty' }, context)
    expect(display.texture).toBeNull()
    expect(display.members).toHaveLength(0)
  })

  it('labels unknown custom tags with a placeholder', () => {
    const display = resolveTagDisplay({ kind: 'custom_tag', uid: 'missing' }, makeTagContext())
    expect(display.label).toBe('#未知标签')
    expect(display.members).toHaveLength(0)
  })
})
