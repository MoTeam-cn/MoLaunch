/**
 * 标签槽位展示解析：成员物品贴图与悬停浮层数据
 */
import type { RecipeSlotContext, SlotValue } from './types'
import { tagLabel } from './tag-zh'

export type TagMember = {
  id: string
  texture: string
  label: string
}

export type TagDisplay = {
  texture: string | null
  label: string
  members: TagMember[]
}

type TagSlotValue = Extract<SlotValue, { kind: 'vanilla_tag' } | { kind: 'custom_tag' }>

function collectMemberIds(value: TagSlotValue, context: RecipeSlotContext): string[] {
  const seen = new Set<string>()
  const ids: string[] = []
  const push = (id: string) => {
    if (!seen.has(id)) {
      seen.add(id)
      ids.push(id)
    }
  }
  if (value.kind === 'vanilla_tag') {
    for (const itemId of context.vanillaTags[value.id] ?? []) push(itemId)
    return ids
  }
  const custom = context.customTagsByUid[value.uid]
  if (!custom) return ids
  for (const member of custom.values) {
    if (member.type === 'item') push(member.id)
    else for (const itemId of context.vanillaTags[member.id] ?? []) push(itemId)
  }
  return ids
}

/** 解析标签槽位展示：首个有贴图成员作图标，全部有贴图成员供悬停浮层使用 */
export function resolveTagDisplay(value: TagSlotValue, context: RecipeSlotContext): TagDisplay {
  const label =
    value.kind === 'vanilla_tag'
      ? `#${tagLabel(value.id)}`
      : `#${context.customTagsByUid[value.uid]?.id ?? '未知标签'}`
  const members: TagMember[] = []
  for (const itemId of collectMemberIds(value, context)) {
    const item = context.itemsById[itemId]
    if (!item?.texture) continue
    members.push({
      id: item.id,
      texture: item.texture,
      label: item.zh ? `${item.name}（${item.zh}）` : item.name,
    })
  }
  return { texture: members[0]?.texture ?? null, label, members }
}
