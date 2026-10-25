/**
 * 合成配方生成器 - 配方编辑状态
 */
import type { RecipeType } from './versions'
import type { RecipeSlot, SlotValue } from './slots'

/** 合成面板选项 */
export type CraftingOptions = {
  shapeless: boolean
  keepWhitespace: boolean
  twoByTwo: boolean
}

/** 烹饪面板选项 */
export type CookingOptions = {
  time: number | null
  experience: number
}

/** 锻造面板选项 */
export type SmithingOptions = {
  trimPattern: string
}

/** 单个配方的完整编辑状态 */
export type RecipeState = {
  id: string
  recipeType: RecipeType
  group: string
  category: string
  showNotification: boolean
  nameMode: 'auto' | 'manual'
  name: string
  slots: Partial<Record<RecipeSlot, SlotValue>>
  crafting: CraftingOptions
  cooking: CookingOptions
  smithing: SmithingOptions
}
