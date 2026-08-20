/**
 * 合成配方槽位 - 槽位值 → 展示信息（图标/标签/数量/标签成员）解析
 */
import { resolveTagDisplay, type TagMember } from '@/utils/recipe-generator/tag-resolve'
import type { RecipeSlotContext, SlotValue } from '@/utils/recipe-generator/types'

export type Display = { texture: string | null; label: string; count: number; members?: TagMember[] }

export function displayFor(value: SlotValue | undefined, context: RecipeSlotContext): Display | null {
  if (!value) return null
  if (value.kind === 'item') {
    const item = context.itemsById[value.id]
    const name = item?.name ?? value.id
    return {
      texture: item?.texture ?? null,
      label: item && item.zh ? `${name}（${item.zh}）` : name,
      count: value.count ?? 1,
    }
  }
  if (value.kind === 'custom_item') {
    const item = context.customItemsByUid[value.uid]
    return {
      texture: item?.texture || null,
      label: item?.name ?? '未知自定义物品',
      count: value.count ?? 1,
    }
  }
  if (value.kind === 'vanilla_tag' || value.kind === 'custom_tag') {
    const display = resolveTagDisplay(value, context)
    return { texture: display.texture, label: display.label, count: 1, members: display.members }
  }
  return null
}

export function barrierDisplayFor(context: RecipeSlotContext): Display | null {
  const item = context.itemsById['minecraft:barrier']
  return item ? { texture: item.texture, label: item.name, count: 1 } : null
}