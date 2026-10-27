/**
 * 方块中文名
 *
 * 复用配方生成器的物品表（自带官方中文名），只取 id → 中文名，不加载贴图信息。
 * 首次查询时才解析，未收录的方块回退英文名。
 */

import rawItems from '@/utils/recipe-generator/assets/items/26.2.json?raw'
import { blockBaseName } from './decode'

interface ItemEntry {
  id: string
  zh?: string
  name?: string
}

let nameMap: Map<string, string> | null = null

function buildNameMap(): Map<string, string> {
  const map = new Map<string, string>()
  try {
    const parsed = JSON.parse(rawItems) as { items?: ItemEntry[] }
    for (const item of parsed.items ?? []) {
      if (!item.id) continue
      const label = item.zh || item.name
      if (label) map.set(item.id, label)
    }
  } catch {
    // 表损坏时退回英文名，不影响预览
  }
  return map
}

function lookup(state: string): Map<string, string> {
  if (!nameMap) nameMap = buildNameMap()
  void state
  return nameMap
}

/** 方块状态 → 中文名；查不到时返回去掉命名空间的英文名 */
export function blockDisplayName(state: string): string {
  const base = blockBaseName(state)
  const map = lookup(base)
  const zh = map.get(base)
  if (zh) return zh
  return base.replace(/^minecraft:/, '')
}
