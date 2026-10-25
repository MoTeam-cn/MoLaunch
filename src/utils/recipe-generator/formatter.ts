/**
 * 合成配方生成器 - 配方 JSON 输出（对应 specs/cleanroom/recipe-generator.md §3 §4）
 *
 * 输出差异集中成三种「引用写法」：legacy(1.12) / object(1.13~1.21.1) / string(1.21.2+)，
 * 其余字段（category、show_notification、pattern、cookingtime）由版本能力表决定。
 */
import {
  DEFAULT_COOKING_TIME,
  getVersionCapability,
  supportsRecipeCategory,
  supportsShowNotification,
  supportsSmithingTrimPattern,
} from './versions'
import type { JavaVersionId, RecipeSlot, RecipeSlotContext, RecipeState, RecipeType, SlotValue } from './types'
import { itemRefToString, parseIdentifier } from './identifier'

/** 引用写法：决定物品/标签怎么写进 JSON */
export type FormatterStrategy = 'legacy' | 'object' | 'id-result' | 'string'

export function getFormatterStrategy(version: JavaVersionId): FormatterStrategy {
  const capability = getVersionCapability(version)
  if (capability.stringIngredients) return 'string'
  if (capability.resultUsesId) return 'id-result'
  if (capability.objectIngredients) return 'object'
  return 'legacy'
}

const SLOT_CAPTIONS: Record<string, string> = {
  ingredient: '原料',
  template: '模板',
  base: '底材',
  addition: '材料',
  result: '产物',
}

/** 槽位中文标题（`smithing.base` → 底材） */
export function slotCaption(slot: RecipeSlot): string {
  const [, tail] = slot.split('.')
  return SLOT_CAPTIONS[tail] ?? tail
}

/** 配方 type 字段：1.14 起带 `minecraft:` 前缀 */
export function recipeTypeFullName(type: RecipeType, shapeless: boolean, version: JavaVersionId): string {
  const bare = type === 'crafting' ? (shapeless ? 'crafting_shapeless' : 'crafting_shaped') : type
  return getVersionCapability(version).namespacedType ? `minecraft:${bare}` : bare
}

/** 槽位 → 物品 ID（内置物品或自定义物品） */
export function resolveItemId(value: SlotValue, context: RecipeSlotContext): string | null {
  if (value.kind === 'item') return value.id
  if (value.kind !== 'custom_item') return null
  const custom = context.customItemsByUid[value.uid]
  if (!custom) return null
  const ref = parseIdentifier(custom.id)
  return ref ? itemRefToString(ref) : null
}

/** 槽位 → 标签 ID（内置标签或自定义标签） */
export function resolveTagId(value: SlotValue, context: RecipeSlotContext): string | null {
  if (value.kind === 'vanilla_tag') return value.id
  if (value.kind !== 'custom_tag') return null
  return context.customTagsByUid[value.uid]?.id ?? null
}

/** 槽位数量（仅物品类有数量概念） */
export function slotCount(value: SlotValue | undefined): number | undefined {
  if (!value) return undefined
  return value.kind === 'item' || value.kind === 'custom_item' ? value.count : undefined
}

/** 槽位值的等价键：同键即可复用同一个 pattern 字符 */
function identityOf(value: SlotValue): string {
  switch (value.kind) {
    case 'item':
      return `item:${value.id}`
    case 'custom_item':
      return `custom_item:${value.uid}`
    case 'vanilla_tag':
      return `vanilla_tag:${value.id}`
    case 'custom_tag':
      return `custom_tag:${value.uid}`
  }
}

export function slotValuesEqual(a: SlotValue, b: SlotValue): boolean {
  return identityOf(a) === identityOf(b)
}

/** 1.12 的物品对象：`{ item, data? }` */
function legacyItemObject(itemId: string): Record<string, unknown> {
  const ref = parseIdentifier(itemId)
  if (!ref) throw new Error('物品标识符无效')
  const out: Record<string, unknown> = { item: itemRefToString({ namespace: ref.namespace, id: ref.id }) }
  if (ref.data !== undefined) out.data = ref.data
  return out
}

/** ingredient 字段（1.21.2+ 允许带 count） */
export function formatIngredient(
  value: SlotValue,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  count?: number,
): unknown {
  const itemId = resolveItemId(value, context)
  if (itemId) {
    if (strategy === 'legacy') return legacyItemObject(itemId)
    if (strategy === 'string') return count !== undefined && count > 1 ? { item: itemId, count } : itemId
    return { item: itemId }
  }

  const tagId = resolveTagId(value, context)
  if (tagId) {
    if (strategy === 'legacy') throw new Error('该版本不支持物品标签')
    if (strategy === 'string') return count !== undefined && count > 1 ? { tag: tagId, count } : `#${tagId}`
    return { tag: tagId }
  }

  throw new Error('槽位内容无效')
}

/** 产物字段名：1.20 起用 `id`，之前用 `item` */
function resultField(strategy: FormatterStrategy): 'item' | 'id' {
  return strategy === 'id-result' || strategy === 'string' ? 'id' : 'item'
}

/** 常规产物（合成 / 锻造转换） */
export function formatResult(
  value: SlotValue,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  count?: number,
): Record<string, unknown> {
  const itemId = resolveItemId(value, context)
  if (!itemId) throw new Error('产物槽位必须是物品')
  const out: Record<string, unknown> = {}
  if (strategy === 'legacy') {
    Object.assign(out, legacyItemObject(itemId))
  } else {
    out[resultField(strategy)] = itemId
  }
  if (count !== undefined && count > 1) out.count = count
  return out
}

/** 熔炼类产物：1.20 前是字符串，之后是 `{ id, count }` */
export function formatCookingResult(
  value: SlotValue,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  count?: number,
): unknown {
  const itemId = resolveItemId(value, context)
  if (!itemId) throw new Error('产物槽位必须是物品')
  if (strategy === 'legacy' || strategy === 'object') return itemId
  const out: Record<string, unknown> = { id: itemId }
  if (count !== undefined && count > 1) out.count = count
  return out
}

/** 切石产物：`{ result: "id" | { id }, count }` */
export function formatStonecutterResult(
  value: SlotValue,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  count?: number,
): Record<string, unknown> {
  const itemId = resolveItemId(value, context)
  if (!itemId) throw new Error('产物槽位必须是物品')
  const out: Record<string, unknown> = {
    result: strategy === 'legacy' || strategy === 'object' ? itemId : { id: itemId },
  }
  if (count !== undefined && count > 1) out.count = count
  return out
}

/** 合成格槽位值（2×2 时只取前四个，行优先） */
export function getCraftingCells(state: RecipeState): (SlotValue | null)[] {
  const size = state.crafting.twoByTwo ? 2 : 3
  const cells: (SlotValue | null)[] = []
  for (let index = 0; index < size * size; index += 1) {
    cells.push(state.slots[`crafting.${index + 1}` as RecipeSlot] ?? null)
  }
  return cells
}

/**
 * 有形状合成：求最小包围盒生成 pattern，并分配 key 字符
 * - `keepWhitespace` 为真时保留完整网格
 * - 全空网格返回 `[' ']`
 */
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
  const byIdentity = new Map<string, string>()
  const keyOf = (value: SlotValue): string => {
    const identity = identityOf(value)
    const known = byIdentity.get(identity)
    if (known) return known
    const char = String.fromCharCode(65 + keys.length)
    byIdentity.set(identity, char)
    keys.push({ char, value })
    return char
  }

  if (state.crafting.keepWhitespace) {
    return {
      pattern: grid.map((row) => row.map((cell) => (cell === null ? ' ' : keyOf(cell))).join('')),
      keys,
    }
  }

  const filledRows = grid.filter((row) => row.some((cell) => cell !== null))
  if (filledRows.length === 0) return { pattern: [' '], keys }

  const filledColumns = filledRows.flatMap((row) =>
    row.flatMap((cell, column) => (cell === null ? [] : [column])),
  )
  const firstColumn = Math.min(...filledColumns)
  const lastColumn = Math.max(...filledColumns)

  return {
    pattern: filledRows.map((row) =>
      row
        .slice(firstColumn, lastColumn + 1)
        .map((cell) => (cell === null ? ' ' : keyOf(cell)))
        .join(''),
    ),
    keys,
  }
}

/** 输入槽位（不含产物） */
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
    case 'stonecutter':
      return ['stonecutter.ingredient']
    case 'smithing':
    case 'smithing_trim':
    case 'smithing_transform':
      return ['smithing.template', 'smithing.base', 'smithing.addition']
    default:
      return ['cooking.ingredient']
  }
}

/** 产物槽位 */
export function getResultSlots(state: RecipeState): RecipeSlot[] {
  switch (state.recipeType) {
    case 'crafting':
      return ['crafting.result']
    case 'stonecutter':
      return ['stonecutter.result']
    case 'smithing':
    case 'smithing_transform':
      return ['smithing.result']
    case 'smithing_trim':
      return []
    default:
      return ['cooking.result']
  }
}

const COOKING_TYPES: readonly RecipeType[] = ['smelting', 'blasting', 'smoking', 'campfire_cooking']

/** 通用头部字段：group / category / show_notification */
function emitHeader(json: Record<string, unknown>, state: RecipeState, version: JavaVersionId): void {
  const smithingType =
    state.recipeType === 'smithing' ||
    state.recipeType === 'smithing_trim' ||
    state.recipeType === 'smithing_transform'

  // 26.1 起切石机不接受 group 字段
  const groupAllowed =
    !smithingType && !(state.recipeType === 'stonecutter' && getVersionCapability(version).notificationCooking)
  if (groupAllowed && state.group) json.group = state.group

  if (state.category && state.category !== 'misc' && supportsRecipeCategory(version, state.recipeType)) {
    json.category = state.category
  }

  if (!state.showNotification && supportsShowNotification(version, state.recipeType, state.crafting.shapeless)) {
    json.show_notification = false
  }
}

/** 合成：有形状写 pattern + key，无形状写扁平 ingredients */
function emitCrafting(
  json: Record<string, unknown>,
  state: RecipeState,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
): void {
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
}

/** 熔炼类：ingredient + result + experience + cookingtime */
function emitCooking(
  json: Record<string, unknown>,
  state: RecipeState,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
): void {
  const ingredient = state.slots['cooking.ingredient']
  if (ingredient) {
    json.ingredient = formatIngredient(ingredient, context, strategy, slotCount(ingredient))
  }

  const result = state.slots['cooking.result']
  if (result) json.result = formatCookingResult(result, context, strategy, slotCount(result))

  json.experience = state.cooking.experience
  const fallback = DEFAULT_COOKING_TIME[state.recipeType as keyof typeof DEFAULT_COOKING_TIME]
  json.cookingtime = state.cooking.time ?? fallback
}

/** 切石机 */
function emitStonecutter(
  json: Record<string, unknown>,
  state: RecipeState,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
): void {
  const ingredient = state.slots['stonecutter.ingredient']
  if (ingredient) {
    json.ingredient = formatIngredient(ingredient, context, strategy, slotCount(ingredient))
  }
  const result = state.slots['stonecutter.result']
  if (result) json.result = formatStonecutterResult(result, context, strategy, slotCount(result))
}

/** 锻造：template / base / addition（+ trim 的 pattern / transform 的 result） */
function emitSmithing(
  json: Record<string, unknown>,
  state: RecipeState,
  context: RecipeSlotContext,
  strategy: FormatterStrategy,
  version: JavaVersionId,
): void {
  const template = state.slots['smithing.template']
  const base = state.slots['smithing.base']
  const addition = state.slots['smithing.addition']
  if (template) json.template = formatIngredient(template, context, strategy, slotCount(template))
  if (base) json.base = formatIngredient(base, context, strategy, slotCount(base))
  if (addition) json.addition = formatIngredient(addition, context, strategy, slotCount(addition))

  if (state.recipeType === 'smithing_trim' && state.smithing.trimPattern && supportsSmithingTrimPattern(version)) {
    json.pattern = state.smithing.trimPattern
  }

  const result = state.slots['smithing.result']
  if (result && state.recipeType !== 'smithing_trim') {
    json.result = formatResult(result, context, strategy, slotCount(result))
  }
}

/** 组装完整配方 JSON */
export function formatRecipeJson(
  state: RecipeState,
  version: JavaVersionId,
  context: RecipeSlotContext,
): Record<string, unknown> {
  const strategy = getFormatterStrategy(version)
  const json: Record<string, unknown> = {
    type: recipeTypeFullName(state.recipeType, state.crafting.shapeless, version),
  }
  emitHeader(json, state, version)

  if (state.recipeType === 'crafting') emitCrafting(json, state, context, strategy)
  else if (COOKING_TYPES.includes(state.recipeType)) emitCooking(json, state, context, strategy)
  else if (state.recipeType === 'stonecutter') emitStonecutter(json, state, context, strategy)
  else emitSmithing(json, state, context, strategy, version)

  return json
}
