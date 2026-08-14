/**
 * 合成配方生成器 - 校验（问题码 + 中文提示）
 */
import type { JavaVersionId, RecipeSlotContext, RecipeState, RecipeType } from './types'
import { isRecipeTypeAvailable, supportsItemTags, supportsSmithingTrimPattern } from './versions'
import { parseIdentifier } from './identifier'
import { getInputSlots, getResultSlots, resolveItemId, resolveTagId } from './formatter'

export const RECIPE_ISSUE_CODES = [
  'unsupported-type',
  'missing-ingredient',
  'missing-result',
  'missing-template',
  'missing-base',
  'missing-addition',
  'missing-trim-pattern',
  'tag-in-result',
  'missing-custom-item',
  'missing-custom-tag',
  'invalid-identifier',
  'tags-not-supported',
] as const

export type RecipeIssueCode = (typeof RECIPE_ISSUE_CODES)[number]

export type RecipeIssue = {
  code: RecipeIssueCode
  message: string
}

const ISSUE_MESSAGES: Record<RecipeIssueCode, string> = {
  'unsupported-type': '该配方类型在此版本中不可用',
  'missing-ingredient': '缺少配方材料',
  'missing-result': '缺少配方产物',
  'missing-template': '缺少锻造模板',
  'missing-base': '缺少锻造基底',
  'missing-addition': '缺少锻造材料',
  'missing-trim-pattern': '缺少纹饰图案',
  'tag-in-result': '结果槽位不能使用标签',
  'missing-custom-item': '引用了不存在的自定义物品',
  'missing-custom-tag': '引用了不存在的自定义标签',
  'invalid-identifier': '标识符格式无效',
  'tags-not-supported': '该版本不支持物品标签',
}

function issue(code: RecipeIssueCode): RecipeIssue {
  return { code, message: ISSUE_MESSAGES[code] }
}

export function validateRecipe(
  state: RecipeState,
  version: JavaVersionId,
  context: RecipeSlotContext,
): RecipeIssue[] {
  const issues: RecipeIssue[] = []

  if (!isRecipeTypeAvailable(version, state.recipeType)) {
    issues.push(issue('unsupported-type'))
    return issues
  }

  const inputSlots = getInputSlots(state)
  const resultSlots = getResultSlots(state)

  for (const slot of inputSlots) {
    const value = state.slots[slot]
    if (!value) {
      if (state.recipeType === 'smithing' || state.recipeType === 'smithing_trim' || state.recipeType === 'smithing_transform') {
        if (slot === 'smithing.template') issues.push(issue('missing-template'))
        else if (slot === 'smithing.base') issues.push(issue('missing-base'))
        else if (slot === 'smithing.addition') issues.push(issue('missing-addition'))
      } else if (state.recipeType === 'crafting') {
        continue
      } else {
        issues.push(issue('missing-ingredient'))
      }
      continue
    }
    issues.push(...validateSlotValue(value, version, context))
  }

  for (const slot of resultSlots) {
    const value = state.slots[slot]
    if (!value) {
      issues.push(issue('missing-result'))
      continue
    }
    if (value.kind === 'vanilla_tag' || value.kind === 'custom_tag') {
      issues.push(issue('tag-in-result'))
    }
    issues.push(...validateSlotValue(value, version, context))
  }

  if (state.recipeType === 'smithing_trim' && supportsSmithingTrimPattern(version) && !state.smithing.trimPattern) {
    issues.push(issue('missing-trim-pattern'))
  }

  if (state.recipeType === 'crafting') {
    const filled = inputSlots.filter((slot) => state.slots[slot]).length
    if (filled < 1) issues.push(issue('missing-ingredient'))
  }

  return dedupeIssues(issues)
}

function validateSlotValue(
  value: RecipeState['slots'][keyof RecipeState['slots']],
  version: JavaVersionId,
  context: RecipeSlotContext,
): RecipeIssue[] {
  if (!value) return []
  switch (value.kind) {
    case 'item': {
      const ref = parseIdentifier(value.id)
      if (!ref) return [issue('invalid-identifier')]
      return []
    }
    case 'vanilla_tag': {
      if (!supportsItemTags(version)) return [issue('tags-not-supported')]
      if (!context.vanillaTags[value.id]) return [issue('invalid-identifier')]
      return []
    }
    case 'custom_item': {
      const custom = context.customItemsByUid[value.uid]
      if (!custom) return [issue('missing-custom-item')]
      if (!parseIdentifier(custom.id)) return [issue('invalid-identifier')]
      return []
    }
    case 'custom_tag': {
      const custom = context.customTagsByUid[value.uid]
      if (!custom) return [issue('missing-custom-tag')]
      if (!supportsItemTags(version)) return [issue('tags-not-supported')]
      return []
    }
  }
}

export function isRecipeValid(
  state: RecipeState,
  version: JavaVersionId,
  context: RecipeSlotContext,
): boolean {
  return validateRecipe(state, version, context).length === 0
}

function dedupeIssues(issues: RecipeIssue[]): RecipeIssue[] {
  return [...new Map(issues.map((item) => [item.code, item])).values()]
}

/** 单个问题的可读展示 */
export function describeRecipeIssues(issues: RecipeIssue[]): string {
  return issues.map((item) => item.message).join('；')
}
