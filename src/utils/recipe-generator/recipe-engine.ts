/**
 * 合成配方生成器 - 配方生成入口
 *
 * 校验通过后按版本策略输出配方 JSON；校验失败抛错（错误信息面向 UI 展示）。
 */
import { formatRecipeJson } from './formatter'
import type { JavaVersionId, RecipeSlot, RecipeSlotContext, RecipeState, SlotValue } from './types'
import { validateRecipe, describeRecipeIssues } from './validation'

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

/** 合成网格槽位值（2x2 时自动裁剪右下角，与 3x3 索引对应） */
export function getCraftingGridValues(state: RecipeState): (SlotValue | undefined)[] {
  const disabled = state.crafting.twoByTwo ? new Set([2, 5, 6, 7, 8]) : new Set<number>()
  return CRAFTING_GRID_SLOTS.map((slot, index) =>
    disabled.has(index) ? undefined : state.slots[slot],
  )
}

/**
 * 生成配方 JSON
 * @throws 校验不通过时抛出带中文提示的 Error
 */
export function generateRecipeJson(
  state: RecipeState,
  version: JavaVersionId,
  context: RecipeSlotContext,
): Record<string, unknown> {
  const issues = validateRecipe(state, version, context)
  if (issues.length > 0) {
    throw new Error(describeRecipeIssues(issues))
  }
  return formatRecipeJson(state, version, context)
}
