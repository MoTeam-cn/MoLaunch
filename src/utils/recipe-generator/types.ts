/**
 * 合成配方生成器 - 类型定义
 */

/** Java 版本（工具支持的 19 个版本，覆盖 1.12 ~ 26.2） */
export const JAVA_VERSIONS = [
  '1.12',
  '1.13',
  '1.14',
  '1.15',
  '1.16',
  '1.17',
  '1.18',
  '1.19',
  '1.20',
  '1.21',
  '1.21.2',
  '1.21.4',
  '1.21.5',
  '1.21.6',
  '1.21.7',
  '1.21.9',
  '1.21.11',
  '26.1',
  '26.2',
] as const

export type JavaVersionId = (typeof JAVA_VERSIONS)[number]

/** 配方类型 */
export const RECIPE_TYPES = [
  'crafting',
  'smelting',
  'blasting',
  'smoking',
  'campfire_cooking',
  'stonecutter',
  'smithing',
  'smithing_trim',
  'smithing_transform',
] as const

export type RecipeType = (typeof RECIPE_TYPES)[number]

/** 数据包 pack.mcmeta 格式号（数组表示 [min_format, max_format]） */
export type PackFormatVersion = number | [number, number]

/** 配方槽位标识 */
export type RecipeSlot =
  | 'crafting.1'
  | 'crafting.2'
  | 'crafting.3'
  | 'crafting.4'
  | 'crafting.5'
  | 'crafting.6'
  | 'crafting.7'
  | 'crafting.8'
  | 'crafting.9'
  | 'crafting.result'
  | 'cooking.ingredient'
  | 'cooking.result'
  | 'stonecutter.ingredient'
  | 'stonecutter.result'
  | 'smithing.template'
  | 'smithing.base'
  | 'smithing.addition'
  | 'smithing.result'

/** 槽位值：内置物品 / 内置标签 / 自定义物品 / 自定义标签 */
export type SlotValue =
  | { kind: 'item'; id: string; count?: number }
  | { kind: 'custom_item'; uid: string; count?: number }
  | { kind: 'vanilla_tag'; id: string }
  | { kind: 'custom_tag'; uid: string }

/** 自定义物品（用户手动录入的任意标识符） */
export type CustomItem = {
  uid: string
  id: string
  name: string
  texture: string
  createdAt: string
}

export type TagValue = {
  type: 'item' | 'tag'
  id: string
}

/** 自定义标签（用户手动定义） */
export type CustomTag = {
  uid: string
  id: string
  values: TagValue[]
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
  crafting: {
    shapeless: boolean
    keepWhitespace: boolean
    twoByTwo: boolean
  }
  cooking: {
    time: number | null
    experience: number
  }
  smithing: {
    trimPattern: string
  }
}

/** 生成 / 校验时所需的解析上下文 */
export type RecipeSlotContext = {
  itemsById: Record<string, { id: string; name: string; zh: string; texture: string | null }>
  customItemsByUid: Record<string, CustomItem>
  customTagsByUid: Record<string, CustomTag>
  vanillaTags: Record<string, string[]>
}

/** 本地存档结构（单配方编辑页用） */
export type RecipeGeneratorStore = {
  version: 1
  selectedVersion: JavaVersionId
  recipes: RecipeState[]
  selectedRecipeId: string
  customItems: CustomItem[]
  customTags: CustomTag[]
}
