/**
 * 合成配方生成器 - 校验（对应 specs/cleanroom/recipe-generator.md §5）
 *
 * 规则化：每条规则独立返回问题码，最后统一去重。
 * 问题码 → 中文提示，界面直接展示 message。
 */
import { isRecipeTypeAvailable, supportsItemTags, supportsSmithingTrimPattern } from './versions'
import { parseIdentifier } from './identifier'
import { getInputSlots, getResultSlots } from './formatter'
import type { JavaVersionId, RecipeSlot, RecipeSlotContext, RecipeState, SlotValue } from './types'

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

const SMITHING_TYPES: readonly string[] = ['smithing', 'smithing_trim', 'smithing_transform']

/** 锻造类的槽位 → 缺失提示码 */
const SMITHING_SLOT_ISSUES: Partial<Record<RecipeSlot, RecipeIssueCode>> = {
  'smithing.template': 'missing-template',
  'smithing.base': 'missing-base',
  'smithing.addition': 'missing-addition',
}

type Rule = (state: RecipeState, version: JavaVersionId, context: RecipeSlotContext) => RecipeIssue[]

/** 单个槽位值的合法性 */
function checkSlotValue(value: SlotValue, version: JavaVersionId, context: RecipeSlotContext): RecipeIssue[] {
  switch (value.kind) {
    case 'item':
      return parseIdentifier(value.id) ? [] : [issue('invalid-identifier')]
    case 'vanilla_tag':
      if (!supportsItemTags(version)) return [issue('tags-not-supported')]
      return context.vanillaTags[value.id] ? [] : [issue('invalid-identifier')]
    case 'custom_item': {
      const custom = context.customItemsByUid[value.uid]
      if (!custom) return [issue('missing-custom-item')]
      return parseIdentifier(custom.id) ? [] : [issue('invalid-identifier')]
    }
    case 'custom_tag': {
      const custom = context.customTagsByUid[value.uid]
      if (!custom) return [issue('missing-custom-tag')]
      return parseIdentifier(custom.id) ? [] : [issue('invalid-identifier')]
    }
  }
}

const typeSupported: Rule = (state, version) =>
  isRecipeTypeAvailable(version, state.recipeType) ? [] : [issue('unsupported-type')]

/** 输入槽位：合成允许留空，锻造缺项逐个提示，其余类型要求必填 */
const inputsFilled: Rule = (state) => {
  const issues: RecipeIssue[] = []
  for (const slot of getInputSlots(state)) {
    if (state.slots[slot]) continue
    if (SMITHING_TYPES.includes(state.recipeType)) {
      const code = SMITHING_SLOT_ISSUES[slot]
      if (code) issues.push(issue(code))
      continue
    }
    if (state.recipeType !== 'crafting') issues.push(issue('missing-ingredient'))
  }
  return issues
}

/** 产物槽位：必填且不允许标签 */
const resultsFilled: Rule = (state) => {
  const issues: RecipeIssue[] = []
  for (const slot of getResultSlots(state)) {
    const value = state.slots[slot]
    if (!value) {
      issues.push(issue('missing-result'))
      continue
    }
    if (value.kind === 'vanilla_tag' || value.kind === 'custom_tag') issues.push(issue('tag-in-result'))
  }
  return issues
}

/** 槽位值本身合法 */
const slotValuesValid: Rule = (state, version, context) => {
  const slots = [...getInputSlots(state), ...getResultSlots(state)]
  return slots.flatMap((slot) => {
    const value = state.slots[slot]
    return value ? checkSlotValue(value, version, context) : []
  })
}

/** 纹饰锻造必须有图案（1.21.5+） */
const trimPatternPresent: Rule = (state, version) =>
  state.recipeType === 'smithing_trim' && supportsSmithingTrimPattern(version) && !state.smithing.trimPattern
    ? [issue('missing-trim-pattern')]
    : []

/** 合成至少放一件材料 */
const craftingNotEmpty: Rule = (state) =>
  state.recipeType === 'crafting' && getInputSlots(state).every((slot) => !state.slots[slot])
    ? [issue('missing-ingredient')]
    : []

const RULES: readonly Rule[] = [
  typeSupported,
  inputsFilled,
  resultsFilled,
  slotValuesValid,
  trimPatternPresent,
  craftingNotEmpty,
]

export function validateRecipe(
  state: RecipeState,
  version: JavaVersionId,
  context: RecipeSlotContext,
): RecipeIssue[] {
  const seen = new Set<RecipeIssueCode>()
  const issues: RecipeIssue[] = []
  for (const rule of RULES) {
    for (const found of rule(state, version, context)) {
      if (seen.has(found.code)) continue
      seen.add(found.code)
      issues.push(found)
    }
  }
  return issues
}

/** 把问题列表拼成一句中文提示（界面 toast 用） */
export function describeRecipeIssues(issues: RecipeIssue[]): string {
  return issues.map((found) => found.message).join('；')
}
