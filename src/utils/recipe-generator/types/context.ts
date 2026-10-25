/**
 * 合成配方生成器 - 解析上下文与本地存档
 */
import type { JavaVersionId } from './versions'
import type { CustomItem, CustomTag } from './slots'
import type { RecipeState } from './recipe'

/** 原版物品表项 */
export type ItemLookup = {
  id: string
  name: string
  zh: string
  texture: string | null
}

/** 生成 / 校验时所需的解析上下文 */
export type RecipeSlotContext = {
  itemsById: Record<string, ItemLookup>
  customItemsByUid: Record<string, CustomItem>
  customTagsByUid: Record<string, CustomTag>
  vanillaTags: Record<string, string[]>
}

/** 本地存档结构 */
export type RecipeGeneratorStore = {
  version: 1
  selectedVersion: JavaVersionId
  recipes: RecipeState[]
  selectedRecipeId: string
  customItems: CustomItem[]
  customTags: CustomTag[]
}
