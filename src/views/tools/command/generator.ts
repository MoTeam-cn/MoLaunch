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

import { ENCHANT_IDS_1_12 } from './data'

/** Minecraft 16 色选项：中文名 + 指令内 id + 预览色值（供 ColorSelect 渲染圆形色块） */
export interface ColorOption {
  label: string
  value: string
  color: string
}

export const COLOR_OPTIONS: ColorOption[] = [
  { label: '黑色', value: 'black', color: '#000000' },
  { label: '深蓝', value: 'dark_blue', color: '#0000AA' },
  { label: '深绿', value: 'dark_green', color: '#00AA00' },
  { label: '深青', value: 'dark_aqua', color: '#00AAAA' },
  { label: '深红', value: 'dark_red', color: '#AA0000' },
  { label: '深紫', value: 'dark_purple', color: '#AA00AA' },
  { label: '金色', value: 'gold', color: '#FFAA00' },
  { label: '灰色', value: 'gray', color: '#AAAAAA' },
  { label: '深灰', value: 'dark_gray', color: '#555555' },
  { label: '蓝色', value: 'blue', color: '#5555FF' },
  { label: '绿色', value: 'green', color: '#55FF55' },
  { label: '青色', value: 'aqua', color: '#55FFFF' },
  { label: '红色', value: 'red', color: '#FF5555' },
  { label: '浅紫', value: 'light_purple', color: '#FF55FF' },
  { label: '黄色', value: 'yellow', color: '#FFFF55' },
  { label: '白色', value: 'white', color: '#FFFFFF' },
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

/** give 指令目标版本选项 */
export const GIVE_VERSIONS: TargetOption[] = [
  { id: '1.20.5', label: '1.20.5+（物品组件）' },
  { id: '1.13', label: '1.13 - 1.20.4（NBT）' },
  { id: '1.12', label: '1.12.2 及以前（数字附魔 ID）' },
]

/** give 指令参数 */
export interface GiveParams {
  itemId: string
  count: number
  target: string
  /** 指令目标版本：'1.20.5' 物品组件 / '1.12' 数字附魔 ID / 其余 1.13 - 1.20.4 NBT */
  version: string
  enchantments: { id: string; lvl: number }[]
  name: string
  nameColor: string
  lore: string[]
  loreColor: string
}

/** 构建 /give 指令（按目标版本输出不同格式） */
export function buildGiveCommand(p: GiveParams): string {
  const item = `minecraft:${p.itemId}`
  const count = p.count > 0 ? ` ${p.count}` : ''
  if (p.version === '1.12') {
    // 1.12.2 及以前：ench 用数字 ID（short，s 后缀），Name/Lore 为纯字符串，数量后需 data 位（0）
    const parts: string[] = []
    const ench = p.enchantments
      .filter((e) => e.id in ENCHANT_IDS_1_12)
      .map((e) => `{id:${ENCHANT_IDS_1_12[e.id]}s,lvl:${e.lvl}s}`)
      .join(',')
    if (ench) parts.push(`ench:[${ench}]`)
    const displayParts: string[] = []
    if (p.name.trim()) {
      displayParts.push(`Name:"${escapeJsonText(p.name.trim())}"`)
    }
    if (p.lore.length > 0) {
      displayParts.push(`Lore:[${p.lore.map((l) => `"${escapeJsonText(l)}"`).join(',')}]`)
    }
    if (displayParts.length > 0) {
      parts.push(`display:{${displayParts.join(',')}}`)
    }
    const nbt = parts.length > 0 ? ` {${parts.join(',')}}` : ''
    return `/give ${p.target} ${item} ${p.count > 0 ? p.count : 1} 0${nbt}`
  }
  if (p.version === '1.20.5') {
    // 1.20.5+ 物品组件格式：方括号内以 minecraft:* 为键，enchantments 用 levels map，键含冒号需引号
    const parts: string[] = []
    if (p.enchantments.length > 0) {
      const levels = p.enchantments.map((e) => `"minecraft:${e.id}":${e.lvl}`).join(',')
      parts.push(`minecraft:enchantments={levels:{${levels}}}`)
    }
    if (p.name.trim()) {
      parts.push(`minecraft:custom_name=${componentSnbt(p.name.trim(), p.nameColor)}`)
    }
    if (p.lore.length > 0) {
      const lore = p.lore.map((l) => componentSnbt(l, p.loreColor)).join(',')
      parts.push(`minecraft:lore=[${lore}]`)
    }
    const components = parts.length > 0 ? `[${parts.join(',')}]` : ''
    return `/give ${p.target} ${item}${components}${count}`
  }
  // 1.13 - 1.20.4 旧 NBT 格式：Enchantments + display.Name + display.Lore
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
  return `/give ${p.target} ${item}${nbt}${count}`
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
