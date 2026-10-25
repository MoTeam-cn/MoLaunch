/**
 * 合成配方生成器 - 版本能力表（对应 specs/cleanroom/recipe-generator.md §3）
 *
 * 每个版本一行，把「配方 JSON 长什么样」的差异集中成布尔开关，
 * 其余模块只读开关，不再散落 `isVersionAtLeast('26.1')` 这类判断。
 */
import { compareVersion } from '@/utils/version'
import type { JavaVersionId, PackFormatVersion, RecipeSlot, RecipeType } from './types'

/** 单个版本的能力开关 */
export type VersionCapability = {
  /** pack.mcmeta 的格式号；null 表示不支持数据包 */
  packFormat: PackFormatVersion | null
  /** 配方目录名；null 表示不支持数据包 */
  recipeDir: 'recipe' | 'recipes' | null
  /** 物品标签目录名；null 表示不支持标签 */
  tagDir: 'tags/item' | 'tags/items' | null
  /** 配方类型写 `minecraft:` 前缀（1.14+） */
  namespacedType: boolean
  /** 物品引用写成 `{ item: "ns:id" }`（1.13~1.21.1）；否则见 stringIngredients */
  objectIngredients: boolean
  /** 物品引用直接写字符串（1.21.2+） */
  stringIngredients: boolean
  /** ingredient 支持 count（1.21.2+） */
  ingredientCount: boolean
  /** 产物字段用 `id`（1.20+），否则用 `item` */
  resultUsesId: boolean
  /** 支持 category 字段（1.19+） */
  category: boolean
  /** 有形状合成支持 show_notification（1.19+） */
  notificationShaped: boolean
  /** 无形状合成支持 show_notification（26.1+） */
  notificationShapeless: boolean
  /** 熔炼/切石/锻造类支持 show_notification（26.1+） */
  notificationCooking: boolean
  /** smithing_trim 支持 pattern 字段（1.21.5+） */
  trimPattern: boolean
  /** 物品标签可用（1.13+） */
  itemTags: boolean
  /** 内置标签清单可用（1.14+） */
  vanillaTags: boolean
}

const LEGACY: VersionCapability = {
  packFormat: null,
  recipeDir: null,
  tagDir: null,
  namespacedType: false,
  objectIngredients: false,
  stringIngredients: false,
  ingredientCount: false,
  resultUsesId: false,
  category: false,
  notificationShaped: false,
  notificationShapeless: false,
  notificationCooking: false,
  trimPattern: false,
  itemTags: false,
  vanillaTags: false,
}

const OBJECT_ERA: VersionCapability = {
  ...LEGACY,
  packFormat: 4,
  recipeDir: 'recipes',
  tagDir: 'tags/items',
  namespacedType: true,
  objectIngredients: true,
  itemTags: true,
}

const TAGGED_ERA: VersionCapability = { ...OBJECT_ERA, vanillaTags: true }

const CATEGORY_ERA: VersionCapability = { ...TAGGED_ERA, category: true, notificationShaped: true }

const ID_RESULT_ERA: VersionCapability = { ...CATEGORY_ERA, resultUsesId: true }

const STRING_ERA: VersionCapability = {
  ...ID_RESULT_ERA,
  recipeDir: 'recipe',
  tagDir: 'tags/item',
  objectIngredients: false,
  stringIngredients: true,
  ingredientCount: true,
}

const TRIM_PATTERN_ERA: VersionCapability = { ...STRING_ERA, trimPattern: true }

const NOTIFICATION_ERA: VersionCapability = {
  ...TRIM_PATTERN_ERA,
  notificationShapeless: true,
  notificationCooking: true,
}

/** 版本 → 能力（数据来源：各版本数据包格式与配方字段的公开变更记录） */
const CAPABILITIES: Record<JavaVersionId, VersionCapability> = {
  '1.12': LEGACY,
  // 1.13 物品写成对象，但配方类型仍是裸名（1.14 起才加 minecraft: 前缀）
  '1.13': { ...OBJECT_ERA, namespacedType: false },
  '1.14': TAGGED_ERA,
  '1.15': { ...TAGGED_ERA, packFormat: 5 },
  '1.16': { ...TAGGED_ERA, packFormat: 6 },
  '1.17': { ...TAGGED_ERA, packFormat: 7 },
  '1.18': { ...TAGGED_ERA, packFormat: 9 },
  '1.19': { ...CATEGORY_ERA, packFormat: 12 },
  '1.20': { ...ID_RESULT_ERA, packFormat: 41 },
  '1.21': { ...STRING_ERA, packFormat: 48 },
  '1.21.2': { ...STRING_ERA, packFormat: 57 },
  '1.21.4': { ...STRING_ERA, packFormat: 61 },
  '1.21.5': { ...TRIM_PATTERN_ERA, packFormat: 71 },
  '1.21.6': { ...TRIM_PATTERN_ERA, packFormat: 80 },
  '1.21.7': { ...TRIM_PATTERN_ERA, packFormat: 81 },
  '1.21.9': { ...TRIM_PATTERN_ERA, packFormat: [88, 0] },
  '1.21.11': { ...TRIM_PATTERN_ERA, packFormat: [94, 1] },
  '26.1': { ...NOTIFICATION_ERA, packFormat: [101, 1] },
  '26.2': { ...NOTIFICATION_ERA, packFormat: [107, 1] },
}

export function getVersionCapability(version: JavaVersionId): VersionCapability {
  return CAPABILITIES[version] ?? LEGACY
}

/** 界面需要的版本元信息 */
export type JavaVersionMeta = {
  id: JavaVersionId
  packFormat: PackFormatVersion | null
  recipeDir: 'recipe' | 'recipes' | null
  tagDir: 'tags/item' | 'tags/items' | null
  hasVanillaTags: boolean
}

/** 版本列表（新版在前） */
export const JAVA_VERSION_LIST: readonly JavaVersionMeta[] = (
  Object.keys(CAPABILITIES) as JavaVersionId[]
)
  .slice()
  .reverse()
  .map((id) => {
    const capability = CAPABILITIES[id]
    return {
      id,
      packFormat: capability.packFormat,
      recipeDir: capability.recipeDir,
      tagDir: capability.tagDir,
      hasVanillaTags: capability.vanillaTags,
    }
  })

export const LATEST_JAVA_VERSION: JavaVersionId = '26.2'

const META_BY_ID = new Map<JavaVersionId, JavaVersionMeta>(JAVA_VERSION_LIST.map((meta) => [meta.id, meta]))

export function getJavaVersionMeta(version: JavaVersionId): JavaVersionMeta {
  const meta = META_BY_ID.get(version)
  if (!meta) throw new Error(`未知的 Java 版本: ${version}`)
  return meta
}

export function isJavaVersionId(value: unknown): value is JavaVersionId {
  return typeof value === 'string' && META_BY_ID.has(value as JavaVersionId)
}

/** 版本号逐段比较（统一走 utils/version 的实现） */
export function compareMinecraftVersions(a: string, b: string): number {
  return compareVersion(a, b)
}

export function isVersionAtLeast(version: JavaVersionId, minimum: string): boolean {
  return compareMinecraftVersions(version, minimum) >= 0
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

/** 配方类型可用区间（min 含、max 仅该版本自身） */
const TYPE_RANGE: Record<RecipeType, { min: string; maxExclusive?: string; maxInclusive?: string }> = {
  crafting: { min: '1.12' },
  smelting: { min: '1.13' },
  blasting: { min: '1.14' },
  smoking: { min: '1.14' },
  campfire_cooking: { min: '1.14' },
  stonecutter: { min: '1.14' },
  smithing: { min: '1.16', maxInclusive: '1.18' },
  smithing_trim: { min: '1.19' },
  smithing_transform: { min: '1.19' },
}

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

export const RECIPE_CATEGORY_LABELS: Record<string, string> = {
  equipment: '装备',
  building: '建筑',
  misc: '杂物',
  redstone: '红石',
  food: '食物',
  blocks: '方块',
}

export function isRecipeTypeAvailable(version: JavaVersionId, type: RecipeType): boolean {
  const range = TYPE_RANGE[type]
  if (!isVersionAtLeast(version, range.min)) return false
  if (range.maxExclusive && isVersionAtLeast(version, range.maxExclusive)) return false
  if (range.maxInclusive && compareMinecraftVersions(version, range.maxInclusive) > 0) return false
  return true
}

export function getSupportedRecipeTypes(version: JavaVersionId): RecipeType[] {
  return ALL_RECIPE_TYPES.filter((type) => isRecipeTypeAvailable(version, type))
}

export function coerceRecipeTypeForVersion(type: RecipeType | undefined, version: JavaVersionId): RecipeType {
  const supported = getSupportedRecipeTypes(version)
  if (type && supported.includes(type)) return type
  return supported[0] ?? 'crafting'
}

const CATEGORY_OPTIONS: Partial<Record<RecipeType, string[]>> = {
  crafting: ['equipment', 'building', 'misc', 'redstone'],
  smelting: ['food', 'blocks', 'misc'],
  blasting: ['blocks', 'misc'],
  smoking: ['food'],
  campfire_cooking: ['food'],
}

export function getRecipeCategoryOptions(type: RecipeType): string[] | undefined {
  return CATEGORY_OPTIONS[type]
}

export function supportsRecipeCategory(version: JavaVersionId, type: RecipeType): boolean {
  return getVersionCapability(version).category && getRecipeCategoryOptions(type) !== undefined
}

export function supportsShowNotification(version: JavaVersionId, type: RecipeType, shapeless: boolean): boolean {
  const capability = getVersionCapability(version)
  if (type === 'crafting') return shapeless ? capability.notificationShapeless : capability.notificationShaped
  if (type === 'smithing') return false
  return capability.notificationCooking
}

export function supportsSmithingTrimPattern(version: JavaVersionId): boolean {
  return getVersionCapability(version).trimPattern
}

export function supportsItemTags(version: JavaVersionId): boolean {
  return getVersionCapability(version).itemTags
}

export function supportsCustomTags(version: JavaVersionId): boolean {
  return getVersionCapability(version).itemTags
}

export function supportsVanillaTagList(version: JavaVersionId): boolean {
  return getVersionCapability(version).vanillaTags
}

export function supportsDatapack(version: JavaVersionId): boolean {
  return getVersionCapability(version).packFormat !== null
}

export const DEFAULT_COOKING_TIME: Record<'smelting' | 'blasting' | 'smoking' | 'campfire_cooking', number> = {
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
