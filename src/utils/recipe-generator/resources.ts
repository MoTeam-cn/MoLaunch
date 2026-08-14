/**
 * 合成配方生成器 - 资源加载
 *
 * 内置数据（MIT 许可）由 scripts/generate-recipe-assets 生成：
 * - assets/items/<version>.json：物品表（id/name/zh/texture）
 * - assets/tags/<version>.json：原版标签（tagId -> 物品 ID 列表）
 * - assets/texture-atlas.png + .json：纹理图集与坐标布局
 *
 * items/tags 按版本懒加载；图集为全量加载。
 */
import type { CustomItem, CustomTag, JavaVersionId, RecipeSlotContext } from './types'

export type AssetItem = {
  id: string
  name: string
  zh: string
  texture: string | null
}

export type VersionItemsManifest = {
  version: string
  items: AssetItem[]
}

export type AtlasLayout = {
  /** 图集总尺寸 [宽, 高] */
  size: [number, number]
  /** 纹理 key -> [x, y, w, h] */
  layout: Record<string, [number, number, number, number]>
}

const itemsGlob = import.meta.glob<VersionItemsManifest>('./assets/items/*.json')
const tagsGlob = import.meta.glob<Record<string, string[]>>('./assets/tags/*.json')

const atlasLayoutPromise = import('./assets/texture-atlas.json').then(
  (m) => m.default as AtlasLayout,
)
const atlasPngUrl = new URL('./assets/texture-atlas.png', import.meta.url).href

const itemsCache = new Map<JavaVersionId, AssetItem[]>()
const tagsCache = new Map<JavaVersionId, Record<string, string[]>>()

export async function loadVersionItems(version: JavaVersionId): Promise<AssetItem[]> {
  const cached = itemsCache.get(version)
  if (cached) return cached
  const mod = itemsGlob[`./assets/items/${version}.json`]
  if (!mod) return []
  const manifest = await mod()
  const items = manifest.items ?? []
  itemsCache.set(version, items)
  return items
}

export async function loadVersionTags(version: JavaVersionId): Promise<Record<string, string[]>> {
  const cached = tagsCache.get(version)
  if (cached) return cached
  const mod = tagsGlob[`./assets/tags/${version}.json`]
  if (!mod) return {}
  const tags = await mod()
  tagsCache.set(version, tags)
  return tags
}

export async function getAtlasLayout(): Promise<AtlasLayout> {
  return atlasLayoutPromise
}

export function getAtlasPngUrl(): string {
  return atlasPngUrl
}

/** 由加载的资源 + 自定义物品/标签组装解析上下文（生成 / 校验用） */
export function buildSlotContext(
  items: AssetItem[],
  tags: Record<string, string[]>,
  customItems: CustomItem[] = [],
  customTags: CustomTag[] = [],
): RecipeSlotContext {
  const itemsById: RecipeSlotContext['itemsById'] = {}
  for (const item of items) itemsById[item.id] = item
  const customItemsByUid: RecipeSlotContext['customItemsByUid'] = {}
  for (const item of customItems) customItemsByUid[item.uid] = item
  const customTagsByUid: RecipeSlotContext['customTagsByUid'] = {}
  for (const tag of customTags) customTagsByUid[tag.uid] = tag
  return { itemsById, customItemsByUid, customTagsByUid, vanillaTags: tags }
}
