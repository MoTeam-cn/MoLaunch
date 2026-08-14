/**
 * 合成配方生成器 - 版本策略格式化器
 *
 * 配方 JSON 随版本演进，按四档策略输出：
 * - legacy(1.12)：{item: "ns:id", data}，type 无命名空间，不支持标签
 * - object(1.13~1.19)：ingredient {item|tag}，result {item, count}，cooking result 为字符串
 * - id-result(1.20~1.21.1)：ingredient {item|tag}，result {id, count}
 * - string(1.21.2+)：ingredient 直接用 "ns:id" / "#ns:tag"
 */
import {
  DEFAULT_COOKING_TIME,
  isVersionAtLeast,
  supportsRecipeCategory,
  supportsShowNotification,
  supportsSmithingTrimPattern,
} from './versions'
import type { JavaVersionId, RecipeSlot, RecipeSlotContext, RecipeState, RecipeType, SlotValue } from './types'
import { itemRefToString, parseIdentifier } from './identifier'

export type FormatterStrategy = 'legacy' | 'object' | 'id-result' | 'string'

export function getFormatterStrategy(version: JavaVersionId): FormatterStrategy {
  if (version === '1.12') return 'legacy'
  if (isVersionAtLeast(version, '1.21.2')) return 'string'
  if (isVersionAtLeast(version, '1.20')) return 'id-result'
  return 'object'
}

/** 配方 type 字段（1.12/1.13 无命名空间前缀） */
export function recipeTypeFullName(
  type: RecipeType,
  shapeless: boolean,
  version: JavaVersionId,
): string {
  const base =
    type === 'crafting' ? (shapeless ? 'crafting_shapeless' : 'crafting_shaped') : type
  if (version === '1.12' || version === '1.13') return base
  return `minecraft:${base}`
}

/** 解析槽位值为物品 ID（内置物品 / 自定义物品） */
export function resolveItemId(value: SlotValue, context: RecipeSlotContext): string | null {
  switch (value.kind) {
    case 'item':
      return value.id
    case 'custom_item': {
      const custom = context.customItemsByUid[value.uid]
      if (!custom) return null
      const ref = parseIdentifier(custom.id)
      return ref ? itemRefToString(ref) : null
    }
    default:
      return null
  }
}

/** 解析槽位值为标签 ID（内置标签 / 自定义标签） */
export function resolveTagId(value: SlotValue, context: RecipeSlotContext): string | null {
  switch (value.kind) {
    case 'vanilla_tag':
      return value.id
    case 'custom_tag': {
      const custom = context.customTagsByUid[value.uid]
      if (!custom) return null
      return custom.id
    }
    default:
      return null
  }
}

/** 取槽位值的数量（仅物品类有） */
export function slotCount(value: SlotValue | undefined): number | undefined {
  return value && (value.kind === 'item' || value.kind === 'custom_item')
    ? value.count
    : undefined
}

const SLOT_CAPTIONS: Record<string, string> = {
  ingredient: '原料',
  template: '模板',
  base: '底材',
  addition: '材料',
  result: '产物',
}

/** 槽位展示标题（如 smithing.base -> 底材） */
export function slotCaption(slot: RecipeSlot): string {
  return SLOT_CAPTIONS[slot.split('.')[1]] ?? slot.split('.')[1]
}

/** 组装 ingredient 字段值（1.21.2+ 输入支持 count） */
export function formatIngredient(
  value: SlotValue,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  count?: number,
): unknown {
  const itemId = resolveItemId(value, context)
  if (itemId) {
    if (strategy === 'string') {
      if (count !== undefined && count > 1) return { item: itemId, count }
      return itemId
    }
    if (strategy === 'legacy') {
      const ref = parseIdentifier(itemId)
      if (!ref) throw new Error('无效的物品标识符')
      const out: Record<string, unknown> = {
        item: itemRefToString({ namespace: ref.namespace, id: ref.id }),
      }
      if (ref.data !== undefined) out.data = ref.data
      return out
    }
    return { item: itemId }
  }
  const tagId = resolveTagId(value, context)
  if (tagId) {
    if (strategy === 'legacy') throw new Error('1.12 不支持标签')
    if (strategy === 'string') {
      if (count !== undefined && count > 1) return { tag: tagId, count }
      return `#${tagId}`
    }
    return { tag: tagId }
  }
  throw new Error('无效的槽位值')
}

/** 常规结果（crafting / smithing 等） */
export function formatResult(
  value: SlotValue,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  count?: number,
): Record<string, unknown> {
  const itemId = resolveItemId(value, context)
  if (!itemId) throw new Error('无效的结果物品')
  const out: Record<string, unknown> = {}
  if (strategy === 'legacy') {
    const ref = parseIdentifier(itemId)
    if (!ref) throw new Error('无效的物品标识符')
    out.item = itemRefToString({ namespace: ref.namespace, id: ref.id })
    if (ref.data !== undefined) out.data = ref.data
  } else if (strategy === 'object') {
    out.item = itemId
  } else {
    out.id = itemId
  }
  if (count !== undefined && count > 1) out.count = count
  return out
}

/** 烹饪结果：1.20 前为字符串 ID，之后为 {id, count} */
export function formatCookingResult(
  value: SlotValue,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  count?: number,
): unknown {
  const itemId = resolveItemId(value, context)
  if (!itemId) throw new Error('无效的结果物品')
  if (strategy === 'legacy' || strategy === 'object') return itemId
  const out: Record<string, unknown> = { id: itemId }
  if (count !== undefined && count > 1) out.count = count
  return out
}

/** 切石结果：1.20 前为 {result: "ns:id", count}，之后为 {result: {id, count}} */
export function formatStonecutterResult(
  value: SlotValue,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  count?: number,
): Record<string, unknown> {
  const itemId = resolveItemId(value, context)
  if (!itemId) throw new Error('无效的结果物品')
  const out: Record<string, unknown> = {}
  if (strategy === 'legacy' || strategy === 'object') {
    out.result = itemId
  } else {
    out.result = { id: itemId }
  }
  if (count !== undefined && count > 1) out.count = count
  return out
}

/** 读取合成网格槽位（3x3 或 2x2，按行优先） */
export function getCraftingCells(state: RecipeState): (SlotValue | null)[] {
  const size = state.crafting.twoByTwo ? 2 : 3
  const cells: (SlotValue | null)[] = []
  for (let index = 0; index < size * size; index += 1) {
    cells.push(state.slots[`crafting.${index + 1}` as RecipeSlot] ?? null)
  }
  return cells
}

/** 计算 pattern（keepWhitespace 时保留完整网格；否则裁剪空行空列），并建立 key -> 槽位值 */
export function buildCraftingPattern(
  state: RecipeState,
  _context: RecipeSlotContext,
): { pattern: string[]; keys: { char: string; value: SlotValue }[] } {
  const size = state.crafting.twoByTwo ? 2 : 3
  const cells = getCraftingCells(state)
  const grid: (SlotValue | null)[][] = Array.from({ length: size }, (_, row) =>
    cells.slice(row * size, row * size + size),
  )

  const keys: { char: string; value: SlotValue }[] = []
  const keyOf = (value: SlotValue): string => {
    const existing = keys.find((entry) => slotValuesEqual(entry.value, value))
    if (existing) return existing.char
    const char = String.fromCharCode(65 + keys.length)
    keys.push({ char, value })
    return char
  }

  if (state.crafting.keepWhitespace) {
    return {
      pattern: grid.map((row) => row.map((cell) => (cell === null ? ' ' : keyOf(cell))).join('')),
      keys,
    }
  }

  const nonEmptyRows = grid.filter((row) => row.some((cell) => cell !== null))
  if (nonEmptyRows.length === 0) return { pattern: [' '], keys }

  let minCol = size
  let maxCol = -1
  for (const row of nonEmptyRows) {
    for (let col = 0; col < row.length; col += 1) {
      if (row[col] !== null) {
        minCol = Math.min(minCol, col)
        maxCol = Math.max(maxCol, col)
      }
    }
  }

  const pattern: string[] = []
  for (const row of nonEmptyRows) {
    pattern.push(
      row
        .slice(minCol, maxCol + 1)
        .map((cell) => (cell === null ? ' ' : keyOf(cell)))
        .join(''),
    )
  }
  return { pattern, keys }
}

export function slotValuesEqual(a: SlotValue, b: SlotValue): boolean {
  if (a.kind !== b.kind) return false
  switch (a.kind) {
    case 'item':
      return b.kind === 'item' && a.id === b.id
    case 'custom_item':
      return b.kind === 'custom_item' && a.uid === b.uid
    case 'vanilla_tag':
      return b.kind === 'vanilla_tag' && a.id === b.id
    case 'custom_tag':
      return b.kind === 'custom_tag' && a.uid === b.uid
  }
}

/** 配方使用的槽位列表（不含结果槽） */
export function getInputSlots(state: RecipeState): RecipeSlot[] {
  switch (state.recipeType) {
    case 'crafting':
      return [
        'crafting.1',
        'crafting.2',
        'crafting.3',
        'crafting.4',
        'crafting.5',
        'crafting.6',
        'crafting.7',
        'crafting.8',
        'crafting.9',
      ]
    case 'smelting':
    case 'blasting':
    case 'smoking':
    case 'campfire_cooking':
      return ['cooking.ingredient']
    case 'stonecutter':
      return ['stonecutter.ingredient']
    case 'smithing':
    case 'smithing_trim':
    case 'smithing_transform':
      return ['smithing.template', 'smithing.base', 'smithing.addition']
  }
}

/** 配方使用的结果槽位 */
export function getResultSlots(state: RecipeState): RecipeSlot[] {
  switch (state.recipeType) {
    case 'crafting':
      return ['crafting.result']
    case 'smelting':
    case 'blasting':
    case 'smoking':
    case 'campfire_cooking':
      return ['cooking.result']
    case 'stonecutter':
      return ['stonecutter.result']
    case 'smithing':
    case 'smithing_transform':
      return ['smithing.result']
    case 'smithing_trim':
      return []
  }
}

export function formatRecipeJson(
  state: RecipeState,
  version: JavaVersionId,
  context: RecipeSlotContext,
): Record<string, unknown> {
  const strategy = getFormatterStrategy(version)
  const json: Record<string, unknown> = {
    type: recipeTypeFullName(state.recipeType, state.crafting.shapeless, version),
  }

  const supportsGroup =
    state.recipeType !== 'smithing' &&
    state.recipeType !== 'smithing_trim' &&
    state.recipeType !== 'smithing_transform'
  if (supportsGroup && state.group) {
    if (!(state.recipeType === 'stonecutter' && isVersionAtLeast(version, '26.1'))) {
      json.group = state.group
    }
  }
  if (state.category && state.category !== 'misc') {
    if (supportsRecipeCategory(version, state.recipeType)) json.category = state.category
  }
  if (!state.showNotification && supportsShowNotification(version, state.recipeType, state.crafting.shapeless)) {
    json.show_notification = false
  }

  if (state.recipeType === 'crafting') {
    if (state.crafting.shapeless) {
      json.ingredients = getCraftingCells(state)
        .filter((cell): cell is SlotValue => cell !== null)
        .map((value) => formatIngredient(value, context, strategy, slotCount(value)))
    } else {
      const { pattern, keys } = buildCraftingPattern(state, context)
      json.pattern = pattern
      json.key = Object.fromEntries(
        keys.map(({ char, value }) => [char, formatIngredient(value, context, strategy, slotCount(value))]),
      )
    }
    const result = state.slots['crafting.result']
    if (result) json.result = formatResult(result, context, strategy, slotCount(result))
  } else if (
    state.recipeType === 'smelting' ||
    state.recipeType === 'blasting' ||
    state.recipeType === 'smoking' ||
    state.recipeType === 'campfire_cooking'
  ) {
    const ingredient = state.slots['cooking.ingredient']
    if (ingredient) {
      json.ingredient = formatIngredient(ingredient, context, strategy, slotCount(ingredient))
    }
    const result = state.slots['cooking.result']
    if (result) {
      json.result = formatCookingResult(result, context, strategy, slotCount(result))
    }
    json.experience = state.cooking.experience
    json.cookingtime = state.cooking.time ?? DEFAULT_COOKING_TIME[state.recipeType]
  } else if (state.recipeType === 'stonecutter') {
    const ingredient = state.slots['stonecutter.ingredient']
    if (ingredient) {
      json.ingredient = formatIngredient(ingredient, context, strategy, slotCount(ingredient))
    }
    const result = state.slots['stonecutter.result']
    if (result) {
      json.result = formatStonecutterResult(result, context, strategy, slotCount(result))
    }
  } else {
    const template = state.slots['smithing.template']
    const base = state.slots['smithing.base']
    const addition = state.slots['smithing.addition']
    if (template) {
      json.template = formatIngredient(template, context, strategy, slotCount(template))
    }
    if (base) json.base = formatIngredient(base, context, strategy, slotCount(base))
    if (addition) {
      json.addition = formatIngredient(addition, context, strategy, slotCount(addition))
    }
    if (
      state.recipeType === 'smithing_trim' &&
      state.smithing.trimPattern &&
      supportsSmithingTrimPattern(version)
    ) {
      json.pattern = state.smithing.trimPattern
    }
    const result = state.slots['smithing.result']
    if (result && state.recipeType !== 'smithing_trim') {
      json.result = formatResult(result, context, strategy, slotCount(result))
    }
  }

  return json
}
