/**
 * 合成配方生成器 - 版本元数据与能力判定
 */
import type { JavaVersionId, PackFormatVersion, RecipeSlot, RecipeType } from './types'

export type JavaVersionMeta = {
  id: JavaVersionId
  packFormat: PackFormatVersion | null
  recipeDir: 'recipe' | 'recipes' | null
  tagDir: 'tags/item' | 'tags/items' | null
  hasVanillaTags: boolean
}

const metadata: Record<JavaVersionId, Omit<JavaVersionMeta, 'id' | 'hasVanillaTags'>> = {
  '1.12': { packFormat: null, recipeDir: null, tagDir: null },
  '1.13': { packFormat: 4, recipeDir: 'recipes', tagDir: 'tags/items' },
  '1.14': { packFormat: 4, recipeDir: 'recipes', tagDir: 'tags/items' },
  '1.15': { packFormat: 5, recipeDir: 'recipes', tagDir: 'tags/items' },
  '1.16': { packFormat: 6, recipeDir: 'recipes', tagDir: 'tags/items' },
  '1.17': { packFormat: 7, recipeDir: 'recipes', tagDir: 'tags/items' },
  '1.18': { packFormat: 9, recipeDir: 'recipes', tagDir: 'tags/items' },
  '1.19': { packFormat: 12, recipeDir: 'recipes', tagDir: 'tags/items' },
  '1.20': { packFormat: 41, recipeDir: 'recipes', tagDir: 'tags/items' },
  '1.21': { packFormat: 48, recipeDir: 'recipe', tagDir: 'tags/item' },
  '1.21.2': { packFormat: 57, recipeDir: 'recipe', tagDir: 'tags/item' },
  '1.21.4': { packFormat: 61, recipeDir: 'recipe', tagDir: 'tags/item' },
  '1.21.5': { packFormat: 71, recipeDir: 'recipe', tagDir: 'tags/item' },
  '1.21.6': { packFormat: 80, recipeDir: 'recipe', tagDir: 'tags/item' },
  '1.21.7': { packFormat: 81, recipeDir: 'recipe', tagDir: 'tags/item' },
  '1.21.9': { packFormat: [88, 0], recipeDir: 'recipe', tagDir: 'tags/item' },
  '1.21.11': { packFormat: [94, 1], recipeDir: 'recipe', tagDir: 'tags/item' },
  '26.1': { packFormat: [101, 1], recipeDir: 'recipe', tagDir: 'tags/item' },
  '26.2': { packFormat: [107, 1], recipeDir: 'recipe', tagDir: 'tags/item' },
}

const TAG_VERSIONS = new Set<JavaVersionId>([
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
])

/** 版本列表（新版在前，界面与导出按此顺序） */
export const JAVA_VERSION_LIST: readonly JavaVersionMeta[] = (
  [
    '26.2',
    '26.1',
    '1.21.11',
    '1.21.9',
    '1.21.7',
    '1.21.6',
    '1.21.5',
    '1.21.4',
    '1.21.2',
    '1.21',
    '1.20',
    '1.19',
    '1.18',
    '1.17',
    '1.16',
    '1.15',
    '1.14',
    '1.13',
    '1.12',
  ] as const
).map((id) => ({
  id,
  ...metadata[id],
  hasVanillaTags: TAG_VERSIONS.has(id),
}))

export const LATEST_JAVA_VERSION: JavaVersionId = '26.2'

const versionById = new Map<JavaVersionId, JavaVersionMeta>(
  JAVA_VERSION_LIST.map((version) => [version.id, version]),
)

export function getJavaVersionMeta(version: JavaVersionId): JavaVersionMeta {
  const meta = versionById.get(version)
  if (!meta) throw new Error(`未知的 Java 版本: ${version}`)
  return meta
}

export function isJavaVersionId(value: unknown): value is JavaVersionId {
  return typeof value === 'string' && versionById.has(value as JavaVersionId)
}

export function compareMinecraftVersions(a: string, b: string): number {
  const aParts = a.split('.').map(Number)
  const bParts = b.split('.').map(Number)
  for (let index = 0; index < Math.max(aParts.length, bParts.length); index += 1) {
    const diff = (aParts[index] ?? 0) - (bParts[index] ?? 0)
    if (diff !== 0) return diff > 0 ? 1 : -1
  }
  return 0
}

export function isVersionAtLeast(version: JavaVersionId, minimum: string): boolean {
  return compareMinecraftVersions(version, minimum) >= 0
}

const recipeTypeAvailability: Record<RecipeType, { minVersion: string; maxVersion?: string }> = {
  crafting: { minVersion: '1.12' },
  smelting: { minVersion: '1.13' },
  blasting: { minVersion: '1.14' },
  smoking: { minVersion: '1.14' },
  campfire_cooking: { minVersion: '1.14' },
  stonecutter: { minVersion: '1.14' },
  smithing: { minVersion: '1.16', maxVersion: '1.18' },
  smithing_trim: { minVersion: '1.19' },
  smithing_transform: { minVersion: '1.19' },
}

export const ALL_RECIPE_TYPES: readonly RecipeType[] = [
  'crafting',
  'smelting',
  'blasting',
  'smoking',
  'campfire_cooking',
  'stonecutter',
  'smithing',
  'smithing_trim',
  'smithing_transform',
]

/** 配方类型中文名（界面下拉展示） */
export const RECIPE_TYPE_LABELS: Record<RecipeType, string> = {
  crafting: '合成',
  smelting: '熔炼',
  blasting: '高炉烧炼',
  smoking: '烟熏',
  campfire_cooking: '营火烹饪',
  stonecutter: '切石',
  smithing: '锻造',
  smithing_trim: '纹饰锻造',
  smithing_transform: '锻造转换',
}

/** 配方分类（category 字段）中文名 */
export const RECIPE_CATEGORY_LABELS: Record<string, string> = {
  equipment: '装备',
  building: '建筑',
  misc: '杂物',
  redstone: '红石',
  food: '食物',
  blocks: '方块',
}

export function isRecipeTypeAvailable(version: JavaVersionId, type: RecipeType): boolean {
  const availability = recipeTypeAvailability[type]
  if (!isVersionAtLeast(version, availability.minVersion)) return false
  if (
    availability.maxVersion &&
    isVersionAtLeast(version, availability.maxVersion) &&
    version !== availability.maxVersion
  ) {
    return false
  }
  return true
}

export function getSupportedRecipeTypes(version: JavaVersionId): RecipeType[] {
  return ALL_RECIPE_TYPES.filter((type) => isRecipeTypeAvailable(version, type))
}

export function coerceRecipeTypeForVersion(
  type: RecipeType | undefined,
  version: JavaVersionId,
): RecipeType {
  const supported = getSupportedRecipeTypes(version)
  if (type && supported.includes(type)) return type
  return supported[0] ?? 'crafting'
}

/** 配方分类（category 字段）可选值 */
export function getRecipeCategoryOptions(type: RecipeType): string[] | undefined {
  switch (type) {
    case 'crafting':
      return ['equipment', 'building', 'misc', 'redstone']
    case 'smelting':
      return ['food', 'blocks', 'misc']
    case 'blasting':
      return ['blocks', 'misc']
    case 'smoking':
    case 'campfire_cooking':
      return ['food']
    default:
      return undefined
  }
}

export function supportsRecipeCategory(version: JavaVersionId, type: RecipeType): boolean {
  return isVersionAtLeast(version, '1.19') && getRecipeCategoryOptions(type) !== undefined
}

export function supportsShowNotification(
  version: JavaVersionId,
  type: RecipeType,
  shapeless: boolean,
): boolean {
  if (type === 'crafting') {
    return isVersionAtLeast(version, shapeless ? '26.1' : '1.19')
  }
  return (
    isVersionAtLeast(version, '26.1') &&
    [
      'smelting',
      'blasting',
      'smoking',
      'campfire_cooking',
      'stonecutter',
      'smithing_trim',
      'smithing_transform',
    ].includes(type)
  )
}

/** smithing_trim 的 pattern 字段（trim 图案）从 1.21.5 起支持 */
export function supportsSmithingTrimPattern(version: JavaVersionId): boolean {
  return isVersionAtLeast(version, '1.21.5')
}

export function supportsItemTags(version: JavaVersionId): boolean {
  return isVersionAtLeast(version, '1.13')
}

export function supportsCustomTags(version: JavaVersionId): boolean {
  return isVersionAtLeast(version, '1.13')
}

export function supportsVanillaTagList(version: JavaVersionId): boolean {
  return isVersionAtLeast(version, '1.14')
}

export const DEFAULT_COOKING_TIME: Record<
  'smelting' | 'blasting' | 'smoking' | 'campfire_cooking',
  number
> = {
  smelting: 200,
  blasting: 100,
  smoking: 100,
  campfire_cooking: 100,
}

export const RESULT_SLOTS_BY_TYPE: Record<RecipeType, readonly RecipeSlot[] | undefined> = {
  crafting: ['crafting.result'],
  smelting: ['cooking.result'],
  blasting: ['cooking.result'],
  smoking: ['cooking.result'],
  campfire_cooking: ['cooking.result'],
  stonecutter: ['stonecutter.result'],
  smithing: ['smithing.result'],
  smithing_trim: undefined,
  smithing_transform: ['smithing.result'],
}

export const SLOT_KEYS_BY_TYPE: Record<RecipeType, readonly RecipeSlot[]> = {
  crafting: [
    'crafting.1',
    'crafting.2',
    'crafting.3',
    'crafting.4',
    'crafting.5',
    'crafting.6',
    'crafting.7',
    'crafting.8',
    'crafting.9',
  ],
  smelting: ['cooking.ingredient'],
  blasting: ['cooking.ingredient'],
  smoking: ['cooking.ingredient'],
  campfire_cooking: ['cooking.ingredient'],
  stonecutter: ['stonecutter.ingredient'],
  smithing: ['smithing.template', 'smithing.base', 'smithing.addition'],
  smithing_trim: ['smithing.template', 'smithing.base', 'smithing.addition'],
  smithing_transform: ['smithing.template', 'smithing.base', 'smithing.addition'],
}
