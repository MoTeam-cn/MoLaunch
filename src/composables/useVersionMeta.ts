/**
 * 版本类型元信息 composable
 *
 * 统一版本类型推断（inferVersionType）与类型映射表（typeMetaMap），
 * 供版本选择页、版本列表、版本设置等多处复用，消除重复实现。
 */

import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'
import anvilIcon from '@/assets/blocks/Anvil.png'
import fabricIcon from '@/assets/blocks/Fabric.png'
import neoforgeIcon from '@/assets/blocks/NeoForge.png'
import optifineIcon from '@/assets/blocks/RedstoneLampOn.png'
import liteloaderIcon from '@/assets/blocks/Egg.png'

export interface VersionTypeMeta {
  icon: string
  label: string
  groupTitle: string
  order: number
}

/**
 * 推断版本类型
 *
 * 优先级（与 PCL2 / 后端 detect_loader_from_json 对齐）：
 *   1. 显式 loader 提示（已知加载器场景，如安装加载器页面）
 *   2. 后端 backendType：后端已经精确分析版本 JSON 的 libraries/字符串内容，
 *      如果返回的是明确的加载器类型（forge/neoforge/fabric/quilt/optifine/liteloader），
 *      必须优先于 ID 关键字匹配——因为整合包版本 ID 可能不含加载器关键字
 *      （例如 "Zombie Invade 100 Days" 不含 forge，但后端 JSON 分析已识别为 Forge）
 *   3. ID 关键字匹配（本地无 backendType 时的兜底）
 *   4. old/fool 等归一化
 *   5. 默认 release
 */
export function inferVersionType(
  versionId: string,
  loader?: string,
  backendType?: string,
): string {
  if (!versionId) return 'release'
  // 1. 显式 loader 提示优先（用于已知加载器的场景）
  if (loader) {
    const l = loader.toLowerCase()
    if (l.includes('neoforge')) return 'neoforge'
    if (l.includes('forge')) return 'forge'
    if (l.includes('fabric')) return 'fabric'
    if (l.includes('optifine')) return 'optifine'
    if (l.includes('liteloader')) return 'liteloader'
  }
  // 2. 后端 backendType 优先于 ID 关键字匹配
  //    后端已精确分析 JSON，前端不应再用 ID 字符串猜测覆盖（如整合包 ID 不含 forge 字样）
  if (backendType) {
    const b = backendType.toLowerCase()
    if (b === 'forge' || b === 'neoforge' || b === 'fabric'
        || b === 'quilt' || b === 'optifine' || b === 'liteloader') {
      return b
    }
    if (b === 'old_beta' || b === 'old_alpha') return 'old'
    if (b === 'fool') return 'fool'
    if (b === 'snapshot') return 'snapshot'
    if (b === 'release') {
      // backendType=release 时仍走 ID 关键字兜底，避免后端漏判（如 ID 明显含 forge 但 JSON 未识别）
      // 落到下面的 ID 匹配
    }
  }
  // 3. ID 关键字匹配（本地兜底）
  const lower = versionId.toLowerCase()
  if (lower.includes('neoforge')) return 'neoforge'
  if (lower.includes('forge')) return 'forge'
  if (lower.includes('fabric')) return 'fabric'
  if (lower.includes('optifine')) return 'optifine'
  if (lower.includes('liteloader')) return 'liteloader'
  if (/^\d{2}w\d{2}[a-z]/.test(versionId)) return 'snapshot'
  // 4. old/fool 归一化（backendType 已在上面处理，这里是纯 ID 兜底）
  // 5. 默认
  return backendType || 'release'
}

/** 版本类型映射表（含 icon + label + groupTitle + order，字段与 VersionSelect 对齐） */
export const typeMetaMap: Record<string, VersionTypeMeta> = {
  forge:      { icon: anvilIcon,        label: 'Forge',      groupTitle: 'Forge 版本',      order: 1 },
  neoforge:   { icon: neoforgeIcon,     label: 'NeoForge',   groupTitle: 'NeoForge 版本',   order: 2 },
  fabric:     { icon: fabricIcon,       label: 'Fabric',     groupTitle: 'Fabric 版本',     order: 3 },
  optifine:   { icon: optifineIcon,     label: 'OptiFine',   groupTitle: 'OptiFine 版本',   order: 4 },
  liteloader: { icon: liteloaderIcon,   label: 'LiteLoader', groupTitle: 'LiteLoader 版本', order: 5 },
  release:    { icon: grassIcon,        label: '正式版',     groupTitle: '原版游戏',        order: 6 },
  snapshot:   { icon: commandBlockIcon, label: '快照',       groupTitle: '快照版本',        order: 7 },
  old:        { icon: cobblestoneIcon,  label: '旧版',       groupTitle: '不常用版本',      order: 8 },
  fool:       { icon: goldBlockIcon,    label: '愚人节版',   groupTitle: '愚人节版本',      order: 9 },
}

/** 从 typeMetaMap 取 icon，未知类型回退到默认图标（草方块） */
export function resolveVersionIcon(type: string): string {
  return typeMetaMap[type]?.icon ?? grassIcon
}

/** 版本类型显示标签（英文短标签，供已安装列表徽标使用） */
export function getVersionTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    release: 'Release',
    snapshot: 'Snapshot',
    forge: 'Forge',
    fabric: 'Fabric',
    neoforge: 'NeoForge',
    optifine: 'OptiFine',
  }
  return labels[type] || type
}

/** 版本类型徽标背景色 class（供已安装列表角标使用） */
export function getVersionTypeBadgeClass(type: string): string {
  switch (type) {
    case 'release': return 'bg-green-500'
    case 'snapshot': return 'bg-yellow-500'
    case 'forge': return 'bg-purple-500'
    case 'fabric': return 'bg-cyan-500'
    case 'neoforge': return 'bg-orange-500'
    case 'optifine': return 'bg-blue-500'
    default: return 'bg-gray-500'
  }
}
