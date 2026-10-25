/**
 * 合成配方生成器 - 生成入口（校验 → 输出）
 *
 * 校验不通过时抛出带中文提示的错误，界面直接 toast。
 */
import { formatRecipeJson } from './formatter'
import { describeRecipeIssues, validateRecipe } from './validation'
import type { JavaVersionId, RecipeSlot, RecipeSlotContext, RecipeState, SlotValue } from './types'

export const CRAFTING_GRID_SLOTS: readonly RecipeSlot[] = [
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

/** 2×2 模式下禁用的槽位下标（3、6、7、8、9 号槽位，0 起算） */
const TWO_BY_TWO_DISABLED = new Set([2, 5, 6, 7, 8])

/** 合成网格槽位值；2×2 时禁用位返回 undefined */
export function getCraftingGridValues(state: RecipeState): (SlotValue | undefined)[] {
  const disabled = state.crafting.twoByTwo ? TWO_BY_TWO_DISABLED : null
  return CRAFTING_GRID_SLOTS.map((slot, index) =>
    disabled?.has(index) ? undefined : state.slots[slot],
  )
}

/**
 * 生成配方 JSON
 * @throws 校验不通过时抛出带中文提示的错误
 */
export function generateRecipeJson(
  state: RecipeState,
  version: JavaVersionId,
  context: RecipeSlotContext,
): Record<string, unknown> {
  const issues = validateRecipe(state, version, context)
  if (issues.length > 0) throw new Error(describeRecipeIssues(issues))
  return formatRecipeJson(state, version, context)
}
