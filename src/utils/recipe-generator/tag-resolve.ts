/**
 * 合成配方生成器 - 标签槽位展示（图标 + 悬停成员列表）
 *
 * 成员来源：内置标签直接查原版标签表；自定义标签先展开其中的物品，
 * 再把其中的标签继续展开，最终按「首个有贴图的成员」作为槽位图标。
 */
import { tagLabel } from './tag-zh'
import type { RecipeSlotContext, SlotValue } from './types'

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

/** 收集标签内的物品 ID（去重保序） */
function memberIdsOf(value: TagSlotValue, context: RecipeSlotContext): string[] {
  const ordered = new Set<string>()
  const addItems = (tagId: string): void => {
    for (const itemId of context.vanillaTags[tagId] ?? []) ordered.add(itemId)
  }

  if (value.kind === 'vanilla_tag') {
    addItems(value.id)
    return [...ordered]
  }

  const custom = context.customTagsByUid[value.uid]
  if (!custom) return []
  for (const member of custom.values) {
    if (member.type === 'item') ordered.add(member.id)
    else addItems(member.id)
  }
  return [...ordered]
}

function labelOf(value: TagSlotValue, context: RecipeSlotContext): string {
  if (value.kind === 'vanilla_tag') return `#${tagLabel(value.id)}`
  return `#${context.customTagsByUid[value.uid]?.id ?? '未知标签'}`
}

export function resolveTagDisplay(value: TagSlotValue, context: RecipeSlotContext): TagDisplay {
  const members: TagMember[] = []
  for (const itemId of memberIdsOf(value, context)) {
    const item = context.itemsById[itemId]
    if (!item?.texture) continue
    members.push({
      id: item.id,
      texture: item.texture,
      label: item.zh ? `${item.name}（${item.zh}）` : item.name,
    })
  }
  return { texture: members[0]?.texture ?? null, label: labelOf(value, context), members }
}
