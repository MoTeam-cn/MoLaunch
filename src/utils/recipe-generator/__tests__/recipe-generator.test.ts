/**
 * 合成配方生成器验收测试（对应 specs/cleanroom/recipe-generator.md §6）
 */
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

/** 直接读资源文件：物品表体积较大，走 JSON.parse 而不是 vite 的 json 插件 */
function readAsset<T>(name: string): T {
  const path = fileURLToPath(new URL(`../assets/${name}`, import.meta.url))
  return JSON.parse(readFileSync(path, 'utf8')) as T
}

const items120 = readAsset<{ items: unknown[] }>('items/1.20.json')
const tags120 = readAsset<Record<string, string[]>>('tags/1.20.json')
import { itemRefToString, parseIdentifier, stripData } from '../identifier'
import {
  coerceRecipeTypeForVersion,
  getJavaVersionMeta,
  getSupportedRecipeTypes,
  isRecipeTypeAvailable,
  isVersionAtLeast,
  supportsRecipeCategory,
  supportsShowNotification,
} from '../versions'
import { formatRecipeJson } from '../formatter'
import { describeRecipeIssues, validateRecipe } from '../validation'
import { createDatapackFiles, createPackMcmeta, sanitizeRecipeName } from '../datapack'
import { generateRecipeJson, getCraftingGridValues } from '../recipe-engine'
import { buildSlotContext } from '../resources'
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

const item = (id: string, count?: number) => ({ kind: 'item' as const, id, ...(count ? { count } : {}) })
const tag = (id: string) => ({ kind: 'vanilla_tag' as const, id })

describe('identifier', () => {
  it('should 解析命名空间与裸标识符 when 输入合法', () => {
    expect(parseIdentifier('minecraft:stone')).toEqual({ namespace: 'minecraft', id: 'stone' })
    expect(parseIdentifier(' stone ')).toEqual({ namespace: 'minecraft', id: 'stone' })
    expect(parseIdentifier('mymod:block/deep')).toEqual({ namespace: 'mymod', id: 'block/deep' })
  })

  it('should 解析 1.12 数据值后缀 when 末尾为数字', () => {
    expect(parseIdentifier('minecraft:wool:14')).toEqual({ namespace: 'minecraft', id: 'wool', data: 14 })
  })

  it('should 拒绝非法标识符 when 字符集或命名空间不合法', () => {
    expect(parseIdentifier('')).toBeNull()
    expect(parseIdentifier('Minecraft:Stone')).toBeNull()
    expect(parseIdentifier('ns:bad id')).toBeNull()
    expect(parseIdentifier('ns@bad:stone')).toBeNull()
  })

  it('should 双向转换 when 使用 itemRefToString 与 stripData', () => {
    expect(itemRefToString({ namespace: 'mymod', id: 'thing' })).toBe('mymod:thing')
    expect(itemRefToString({ namespace: 'minecraft', id: 'wool', data: 14 })).toBe('minecraft:wool:14')
    expect(stripData('minecraft:wool:14')).toBe('minecraft:wool')
  })
})

describe('versions', () => {
  it('should 逐段比较版本号 when 段数不同', () => {
    expect(isVersionAtLeast('1.21.2', '1.21')).toBe(true)
    expect(isVersionAtLeast('1.12', '1.13')).toBe(false)
    expect(isVersionAtLeast('26.1', '1.21.11')).toBe(true)
  })

  it('should 按版本开关配方类型 when 查询可用类型', () => {
    expect(isRecipeTypeAvailable('1.12', 'smelting')).toBe(false)
    expect(isRecipeTypeAvailable('1.13', 'smelting')).toBe(true)
    expect(isRecipeTypeAvailable('1.18', 'smithing')).toBe(true)
    expect(isRecipeTypeAvailable('1.19', 'smithing')).toBe(false)
    expect(isRecipeTypeAvailable('1.19', 'smithing_transform')).toBe(true)
  })

  it('should 回退到首个可用类型 when 类型不被版本支持', () => {
    expect(getSupportedRecipeTypes('1.12')).toEqual(['crafting'])
    expect(coerceRecipeTypeForVersion('smithing', '1.12')).toBe('crafting')
    expect(coerceRecipeTypeForVersion('crafting', '1.20')).toBe('crafting')
  })

  it('should 暴露数据包元信息 when 读取版本元数据', () => {
    expect(getJavaVersionMeta('1.12').packFormat).toBeNull()
    expect(getJavaVersionMeta('1.20').recipeDir).toBe('recipes')
    expect(getJavaVersionMeta('1.21').recipeDir).toBe('recipe')
    expect(getJavaVersionMeta('1.21.9').packFormat).toEqual([88, 0])
  })

  it('should 按 1.19 与 26.1 划界 when 判断 category 与 show_notification', () => {
    expect(supportsRecipeCategory('1.18', 'crafting')).toBe(false)
    expect(supportsRecipeCategory('1.19', 'crafting')).toBe(true)
    expect(supportsRecipeCategory('1.20', 'stonecutter')).toBe(false)
    expect(supportsShowNotification('1.18', 'crafting', false)).toBe(false)
    expect(supportsShowNotification('1.19', 'crafting', false)).toBe(true)
    expect(supportsShowNotification('1.20', 'crafting', true)).toBe(false)
    expect(supportsShowNotification('26.1', 'crafting', true)).toBe(true)
    expect(supportsShowNotification('1.20', 'smelting', false)).toBe(false)
    expect(supportsShowNotification('26.1', 'smelting', false)).toBe(true)
    expect(supportsShowNotification('26.1', 'smithing', false)).toBe(false)
  })
})

describe('formatter', () => {
  it('should 输出 pattern 与 key when 1.20 有形状合成', () => {
    const state = makeState({
      slots: { 'crafting.1': item('minecraft:oak_planks'), 'crafting.2': item('minecraft:oak_planks'), 'crafting.result': item('minecraft:stick', 4) },
    })
    expect(formatRecipeJson(state, '1.20', makeContext())).toEqual({
      type: 'minecraft:crafting_shaped',
      pattern: ['AA'],
      key: { A: { item: 'minecraft:oak_planks' } },
      result: { id: 'minecraft:stick', count: 4 },
    })
  })

  it('should 把数量写进 ingredient when 1.21.2+', () => {
    const state = makeState({
      slots: { 'crafting.1': item('minecraft:oak_planks', 2), 'crafting.result': item('minecraft:stick') },
    })
    const json = formatRecipeJson(state, '1.21.2', makeContext()) as Record<string, unknown>
    expect(json.key).toEqual({ A: { item: 'minecraft:oak_planks', count: 2 } })
  })

  it('should 省略 ingredient 数量 when 版本低于 1.21.2', () => {
    const state = makeState({
      slots: { 'crafting.1': item('minecraft:oak_planks', 2), 'crafting.result': item('minecraft:stick') },
    })
    const json = formatRecipeJson(state, '1.20', makeContext()) as Record<string, unknown>
    expect(json.key).toEqual({ A: { item: 'minecraft:oak_planks' } })
  })

  it('should 输出 legacy 结构 when 1.12 有形状合成', () => {
    const state = makeState({
      slots: { 'crafting.1': item('minecraft:wool:14'), 'crafting.result': item('minecraft:stick', 2) },
    })
    expect(formatRecipeJson(state, '1.12', makeContext())).toEqual({
      type: 'crafting_shaped',
      pattern: ['A'],
      key: { A: { item: 'minecraft:wool', data: 14 } },
      result: { item: 'minecraft:stick', count: 2 },
    })
  })

  it('should 输出扁平 ingredients when 1.21 无形状合成', () => {
    const state = makeState({
      crafting: { shapeless: true, keepWhitespace: false, twoByTwo: false },
      slots: { 'crafting.1': item('minecraft:oak_planks'), 'crafting.5': item('minecraft:stick'), 'crafting.result': item('minecraft:crafting_table') },
    })
    expect(formatRecipeJson(state, '1.21', makeContext())).toEqual({
      type: 'minecraft:crafting_shapeless',
      ingredients: ['minecraft:oak_planks', 'minecraft:stick'],
      result: { id: 'minecraft:crafting_table' },
    })
  })

  it('should 输出字符串标签引用 when 1.21.2+ 使用标签', () => {
    const state = makeState({
      crafting: { shapeless: true, keepWhitespace: false, twoByTwo: false },
      slots: { 'crafting.1': tag('minecraft:planks'), 'crafting.result': item('minecraft:stick') },
    })
    const json = formatRecipeJson(state, '1.21.2', makeContext()) as Record<string, unknown>
    expect(json.ingredients).toEqual(['#minecraft:planks'])
  })

  it('should 按 1.20 划界切换熔炼产物结构 when 生成熔炼配方', () => {
    const state = makeState({
      recipeType: 'smelting',
      slots: { 'cooking.ingredient': item('minecraft:raw_iron'), 'cooking.result': item('minecraft:iron_ingot') },
    })
    const legacy = formatRecipeJson(state, '1.19', makeContext()) as Record<string, unknown>
    expect(legacy.result).toBe('minecraft:iron_ingot')
    expect(legacy.cookingtime).toBe(200)
    const modern = formatRecipeJson(state, '1.20', makeContext()) as Record<string, unknown>
    expect(modern.result).toEqual({ id: 'minecraft:iron_ingot' })
  })

  it('should 嵌套切石产物 when 1.20 及以上', () => {
    const state = makeState({
      recipeType: 'stonecutter',
      slots: { 'stonecutter.ingredient': item('minecraft:stone'), 'stonecutter.result': item('minecraft:stone_slab', 2) },
    })
    const old = formatRecipeJson(state, '1.19', makeContext()) as { result: Record<string, unknown> }
    expect(old.result.result).toBe('minecraft:stone_slab')
    expect(old.result.count).toBe(2)
    const modern = formatRecipeJson(state, '1.20', makeContext()) as { result: Record<string, unknown> }
    expect(modern.result.result).toEqual({ id: 'minecraft:stone_slab' })
    expect(modern.result.count).toBe(2)
  })

  it('should 仅在支持且关闭时写 show_notification when 生成配方', () => {
    const state = makeState({
      showNotification: false,
      slots: { 'crafting.1': item('minecraft:oak_planks'), 'crafting.result': item('minecraft:stick') },
    })
    expect(formatRecipeJson(state, '1.18', makeContext()).show_notification).toBeUndefined()
    expect(formatRecipeJson(state, '1.20', makeContext()).show_notification).toBe(false)
  })

  it('should 不写产物并写 pattern when 1.21.5 纹饰锻造', () => {
    const state = makeState({
      recipeType: 'smithing_trim',
      smithing: { trimPattern: 'minecraft:sentry' },
      slots: {
        'smithing.template': item('minecraft:sentry_armor_trim_smithing_template'),
        'smithing.base': item('minecraft:diamond_chestplate'),
        'smithing.addition': item('minecraft:emerald'),
        'smithing.result': item('minecraft:diamond_chestplate'),
      },
    })
    const json = formatRecipeJson(state, '1.21.5', makeContext()) as Record<string, unknown>
    expect(json.pattern).toBe('minecraft:sentry')
    expect(json.result).toBeUndefined()
    expect(json.template).toBe('minecraft:sentry_armor_trim_smithing_template')
  })

  it('should 按 1.14 划界加命名空间前缀 when 生成 type', () => {
    const state = makeState({
      slots: { 'crafting.1': item('minecraft:oak_planks'), 'crafting.result': item('minecraft:stick') },
    })
    expect(formatRecipeJson(state, '1.13', makeContext()).type).toBe('crafting_shaped')
    expect(formatRecipeJson(state, '1.14', makeContext()).type).toBe('minecraft:crafting_shaped')
  })
})

describe('validation', () => {
  it('should 报告缺少材料与产物 when 配方不完整', () => {
    const issues = validateRecipe(makeState({ recipeType: 'smelting' }), '1.20', makeContext())
    expect(issues.map((found) => found.code)).toEqual(['missing-ingredient', 'missing-result'])
    expect(describeRecipeIssues(issues)).toContain('缺少配方材料')
  })

  it('should 拒绝结果槽位使用标签 when 产物为标签', () => {
    const state = makeState({ slots: { 'crafting.1': item('minecraft:oak_planks'), 'crafting.result': tag('minecraft:planks') } })
    expect(validateRecipe(state, '1.20', makeContext()).map((found) => found.code)).toContain('tag-in-result')
  })

  it('should 拒绝 1.12 的标签 when 版本不支持标签', () => {
    const state = makeState({ slots: { 'crafting.1': tag('minecraft:planks'), 'crafting.result': item('minecraft:stick') } })
    expect(validateRecipe(state, '1.12', makeContext()).map((found) => found.code)).toContain('tags-not-supported')
  })

  it('should 要求纹饰图案 when 1.21.5 的 smithing_trim 未填图案', () => {
    const state = makeState({
      recipeType: 'smithing_trim',
      slots: {
        'smithing.template': item('minecraft:sentry_armor_trim_smithing_template'),
        'smithing.base': item('minecraft:diamond_chestplate'),
        'smithing.addition': item('minecraft:emerald'),
      },
    })
    expect(validateRecipe(state, '1.21.5', makeContext()).map((found) => found.code)).toContain('missing-trim-pattern')
    expect(validateRecipe(state, '1.21.4', makeContext()).map((found) => found.code)).not.toContain('missing-trim-pattern')
  })

  it('should 无问题时通过 when 配方完整', () => {
    const state = makeState({ slots: { 'crafting.1': item('minecraft:oak_planks'), 'crafting.result': item('minecraft:stick') } })
    expect(validateRecipe(state, '1.20', makeContext())).toEqual([])
  })
})

describe('datapack', () => {
  it('should 安全化文件名 when 名称含非法字符', () => {
    expect(sanitizeRecipeName(' My Recipe! ', 'recipe')).toBe('my_recipe')
    expect(sanitizeRecipeName('***', 'fallback')).toBe('fallback')
  })

  it('should 写 pack_format 或区间 when 版本不同', () => {
    expect(JSON.parse(createPackMcmeta(48)).pack.pack_format).toBe(48)
    const ranged = JSON.parse(createPackMcmeta([101, 1])).pack
    expect(ranged.min_format).toBe(101)
    expect(ranged.max_format).toBe(1)
  })

  it('should 按版本切换配方目录 when 打包文件', () => {
    const recipe = { name: 'demo', json: { type: 'minecraft:crafting_shaped' } }
    expect(createDatapackFiles('1.20', [recipe], []).map((file) => file.path)).toContain('data/crafting/recipes/demo.json')
    expect(createDatapackFiles('1.21', [recipe], []).map((file) => file.path)).toContain('data/crafting/recipe/demo.json')
  })

  it('should 把自定义标签写到标签目录 when 打包文件', () => {
    const files = createDatapackFiles('1.21', [], [{ namespace: 'mymod', id: 'wood', values: ['minecraft:oak_planks'] }])
    const tagFile = files.find((file) => file.path === 'data/mymod/tags/item/wood.json')
    expect(tagFile).toBeDefined()
    expect(JSON.parse(tagFile?.content ?? '{}').values).toEqual(['minecraft:oak_planks'])
  })

  it('should 抛错 when 版本不支持数据包', () => {
    expect(() => createDatapackFiles('1.12', [], [])).toThrow('不支持数据包')
  })
})

describe('recipe-engine', () => {
  it('should 禁用 2×2 的边角槽位 when 开启 twoByTwo', () => {
    const state = makeState({
      crafting: { shapeless: false, keepWhitespace: false, twoByTwo: true },
      slots: { 'crafting.1': item('a:one'), 'crafting.3': item('a:three'), 'crafting.4': item('a:four') },
    })
    const values = getCraftingGridValues(state)
    expect(values).toHaveLength(9)
    expect(values[0]).toBeDefined()
    expect(values[1]).toBeUndefined()
    expect(values[2]).toBeUndefined()
    expect(values[3]).toBeDefined()
    expect(values.slice(4).every((value) => value === undefined)).toBe(true)
  })

  it('should 生成配方 JSON when 配方合法', () => {
    const state = makeState({ slots: { 'crafting.1': item('minecraft:oak_planks'), 'crafting.result': item('minecraft:stick') } })
    expect(generateRecipeJson(state, '1.20', makeContext()).type).toBe('minecraft:crafting_shaped')
  })

  it('should 抛出中文提示 when 配方非法', () => {
    expect(() => generateRecipeJson(makeState({ recipeType: 'smelting' }), '1.20', makeContext())).toThrow('缺少配方材料')
  })
})

describe('resources', () => {
  it('should 组装解析上下文 when 传入物品与标签', () => {
    const context = buildSlotContext(
      [{ id: 'minecraft:stone', name: 'Stone', zh: '石头', texture: 'stone' }],
      { 'minecraft:stone': ['minecraft:stone'] },
      [{ uid: 'item-1', id: 'mymod:thing', name: 'Thing', texture: 'thing', createdAt: '2026-01-01' }],
      [{ uid: 'tag-1', id: 'mymod:group', values: [{ type: 'item', id: 'minecraft:stone' }] }],
    )
    expect(context.itemsById['minecraft:stone'].zh).toBe('石头')
    expect(context.customItemsByUid['item-1'].id).toBe('mymod:thing')
    expect(context.customTagsByUid['tag-1'].values).toHaveLength(1)
    expect(context.vanillaTags['minecraft:stone']).toEqual(['minecraft:stone'])
  })

  it('should 提供版本物品与标签数据 when 读取内置资源', () => {
    expect(items120.items.length).toBeGreaterThan(100)
    expect(Object.keys(tags120).length).toBeGreaterThan(10)
  })
})

describe('tag-zh', () => {
  it('should 为内置标签提供中文名 when 查询已知标签', () => {
    for (const tagId of Object.keys(tags120)) {
      expect(tagLabel(tagId)).not.toBe(tagId)
    }
  })

  it('should 回退为可读名称 when 标签未知', () => {
    expect(tagLabel('mymod:custom_tag')).toBe('Mymod:custom Tag')
  })
})

describe('tag-resolve', () => {
  function makeTagContext(): RecipeSlotContext {
    return {
      itemsById: {
        'minecraft:oak_planks': { id: 'minecraft:oak_planks', name: 'Oak Planks', zh: '橡木木板', texture: 'oak_planks' },
        'minecraft:spruce_planks': { id: 'minecraft:spruce_planks', name: 'Spruce Planks', zh: '云杉木板', texture: 'spruce_planks' },
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
      vanillaTags: { 'minecraft:planks': ['minecraft:oak_planks', 'minecraft:spruce_planks', 'minecraft:air'] },
    }
  }

  it('should 取首个有贴图成员作为图标 when 内置标签', () => {
    const display = resolveTagDisplay({ kind: 'vanilla_tag', id: 'minecraft:planks' }, makeTagContext())
    expect(display.label).toBe(`#${tagLabel('minecraft:planks')}`)
    expect(display.texture).toBe('oak_planks')
    expect(display.members.map((member) => member.id)).toEqual(['minecraft:oak_planks', 'minecraft:spruce_planks'])
  })

  it('should 展开嵌套标签并去重 when 自定义标签', () => {
    const display = resolveTagDisplay({ kind: 'custom_tag', uid: 'tag-1' }, makeTagContext())
    expect(display.label).toBe('#mymod:wood')
    expect(display.texture).toBe('oak_planks')
    const ids = display.members.map((member) => member.id)
    expect(ids).toContain('minecraft:oak_planks')
    expect(ids).toContain('minecraft:spruce_planks')
    expect(ids).not.toContain('minecraft:air')
    expect(ids).toHaveLength(2)
  })

  it('should 返回空贴图 when 成员都没有贴图', () => {
    const context = makeTagContext()
    context.vanillaTags['minecraft:empty'] = ['minecraft:air']
    const display = resolveTagDisplay({ kind: 'vanilla_tag', id: 'minecraft:empty' }, context)
    expect(display.texture).toBeNull()
    expect(display.members).toHaveLength(0)
  })

  it('should 用占位名 when 自定义标签不存在', () => {
    const display = resolveTagDisplay({ kind: 'custom_tag', uid: 'missing' }, makeTagContext())
    expect(display.label).toBe('#未知标签')
    expect(display.members).toHaveLength(0)
  })
})
