/**
 * 合成配方生成器 - 槽位与槽位值
 *
 * 槽位名由「面板.槽位」拼成：合成网格 3×3 + 产物、烹饪、切石机、锻造。
 */

/** 合成面板槽位 */
const CRAFTING_CELLS = ['1', '2', '3', '4', '5', '6', '7', '8', '9', 'result'] as const
/** 烹饪面板槽位（熔炉 / 烟熏炉 / 营火） */
const COOKING_CELLS = ['ingredient', 'result'] as const
/** 切石机槽位 */
const STONECUTTER_CELLS = ['ingredient', 'result'] as const
/** 锻造面板槽位 */
const SMITHING_CELLS = ['template', 'base', 'addition', 'result'] as const

/** 配方槽位标识 */
export type RecipeSlot =
  | `crafting.${(typeof CRAFTING_CELLS)[number]}`
  | `cooking.${(typeof COOKING_CELLS)[number]}`
  | `stonecutter.${(typeof STONECUTTER_CELLS)[number]}`
  | `smithing.${(typeof SMITHING_CELLS)[number]}`

/** 槽位值：内置物品 / 内置标签 / 自定义物品 / 自定义标签 */
export type SlotValue =
  | { kind: 'item'; id: string; count?: number }
  | { kind: 'custom_item'; uid: string; count?: number }
  | { kind: 'vanilla_tag'; id: string }
  | { kind: 'custom_tag'; uid: string }

/** 标签引用（type 决定查原版标签表还是自定义标签） */
export type TagValue = {
  type: 'item' | 'tag'
  id: string
}

/** 用户手动录入的物品 */
export type CustomItem = {
  uid: string
  id: string
  name: string
  texture: string
  createdAt: string
}

/** 用户手动定义的标签 */
export type CustomTag = {
  uid: string
  id: string
  values: TagValue[]
}
