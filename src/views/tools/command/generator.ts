/**
 * 指令生成工具 - 纯前端 SNBT / 指令生成器
 *
 * 生成 Minecraft Java 版（1.20.5+ 兼容 1.13+ 旧格式）常用指令：
 * - give：物品编辑（附魔 / 自定义名称 / Lore）
 * - setblock：放置告示牌（告示牌商店）
 * - summon：召唤实体
 *
 * SNBT 规则要点：
 * - JSON 文本组件整体作为 SNBT 字符串，外层用单引号包裹：
 *   Name:'{"text":"你好","color":"gold"}'
 * - 文本组件内部 JSON 双引号需转义为 \"，SNBT 字符串内单引号需转义为 \'
 */

/** Minecraft 标准 16 色名称 */
export const MC_COLORS = [
  'black',
  'dark_blue',
  'dark_green',
  'dark_aqua',
  'dark_red',
  'dark_purple',
  'gold',
  'gray',
  'dark_gray',
  'blue',
  'green',
  'aqua',
  'red',
  'light_purple',
  'yellow',
  'white',
]

/** JSON 字符串转义（用于文本组件内部） */
export function escapeJsonText(text: string): string {
  return text
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '')
    .replace(/\t/g, '\\t')
}

/** SNBT 单引号字符串（包裹 JSON 组件等），转义内部单引号 */
export function snbtQuote(value: string): string {
  return `'${value.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}'`
}

/** 生成 Minecraft 文本组件 JSON 字符串（如 {"text":"你好","color":"gold"}） */
export function textComponent(text: string, color = ''): string {
  const parts: string[] = [`"text":"${escapeJsonText(text)}"`]
  if (color) parts.push(`"color":"${color}"`)
  return `{${parts.join(',')}}`
}

/** 将文本组件 JSON 包装为 SNBT 单引号字符串 */
export function componentSnbt(text: string, color = ''): string {
  return snbtQuote(textComponent(text, color))
}

/** 玩家选择器 */
export interface TargetOption {
  id: string
  label: string
}

export const TARGETS: TargetOption[] = [
  { id: '@p', label: '@p 最近玩家' },
  { id: '@a', label: '@a 全部玩家' },
  { id: '@s', label: '@s 执行者' },
  { id: '@r', label: '@r 随机玩家' },
]

/** give 指令参数 */
export interface GiveParams {
  itemId: string
  count: number
  target: string
  enchantments: { id: string; lvl: number }[]
  name: string
  nameColor: string
  lore: string[]
  loreColor: string
}

/** 构建 /give 指令（NBT 部分：Enchantments + display.Name + display.Lore） */
export function buildGiveCommand(p: GiveParams): string {
  const nbtParts: string[] = []
  if (p.enchantments.length > 0) {
    const ench = p.enchantments
      .map((e) => `{id:"minecraft:${e.id}",lvl:${e.lvl}}`)
      .join(',')
    nbtParts.push(`Enchantments:[${ench}]`)
  }
  const displayParts: string[] = []
  if (p.name.trim()) {
    displayParts.push(`Name:${componentSnbt(p.name.trim(), p.nameColor)}`)
  }
  if (p.lore.length > 0) {
    const lore = p.lore.map((l) => componentSnbt(l, p.loreColor)).join(',')
    displayParts.push(`Lore:[${lore}]`)
  }
  if (displayParts.length > 0) {
    nbtParts.push(`display:{${displayParts.join(',')}}`)
  }
  const nbt = nbtParts.length > 0 ? `{${nbtParts.join(',')}}` : ''
  const count = p.count > 0 ? ` ${p.count}` : ''
  return `/give ${p.target} minecraft:${p.itemId}${nbt}${count}`
}

/** 告示牌商店指令参数 */
export interface SignShopParams {
  signId: string
  facing: string
  x: string
  y: string
  z: string
  lines: string[]
  textColor: string
}

/** 构建告示牌四行 messages 数组（front_text.messages，恰好 4 条） */
function buildSignMessages(lines: string[], textColor: string): string {
  const padded = [...lines, '', '', '', ''].slice(0, 4)
  const messages = padded.map((l) => componentSnbt(l, textColor)).join(',')
  return `messages:[${messages}]`
}

/** 构建 /setblock 放置告示牌（含 front_text 文本），用于告示牌商店 */
export function buildSignShopCommand(p: SignShopParams): string {
  const frontText = `front_text:{${buildSignMessages(p.lines, p.textColor)}}`
  const coords = `${p.x || '~'} ${p.y || '~'} ${p.z || '~'}`
  return `/setblock ${coords} minecraft:${p.signId}[facing=${p.facing}]{${frontText}} replace`
}

/** summon 指令参数 */
export interface SummonParams {
  entityId: string
  x: string
  y: string
  z: string
  name: string
  nameColor: string
  count: number
}

/** 构建 /summon 指令（支持数量、自定义名称） */
export function buildSummonCommand(p: SummonParams): string {
  const nbtParts: string[] = []
  if (p.name.trim()) {
    nbtParts.push(`CustomName:${componentSnbt(p.name.trim(), p.nameColor)}`)
    nbtParts.push('CustomNameVisible:1b')
  }
  const nbt = nbtParts.length > 0 ? `{${nbtParts.join(',')}}` : ''
  const coords = `${p.x || '~'} ${p.y || '~'} ${p.z || '~'}`
  const count = p.count > 1 ? ` ${p.count}` : ''
  return `/summon minecraft:${p.entityId} ${coords}${nbt}${count}`
}
