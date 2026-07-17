/**
 * Mod 显示辅助函数（从 ModTab.vue 抽出）
 *
 * 纯展示函数：根据 modLocalNameStyle 返回标题/副标题，加载器类型可视化，
 * 以及为百科搜索去除 mod 名称中的版本号。
 */
import type { ModInfo } from '@/utils/tauri'

/** 根据 modLocalNameStyle 返回 Mod 标题（主显示名） */
export function modTitle(mod: ModInfo, style: number): string {
  if (style === 0) {
    return mod.translated_name || mod.enabled_name
  }
  return mod.enabled_name
}

/** 根据 modLocalNameStyle 返回 Mod 副标题（详情名） */
export function modSubtitle(mod: ModInfo, style: number): string {
  if (style === 0) {
    return mod.enabled_name
  }
  return mod.translated_name
}

/** 加载器类型对应的显示标签（用于详情行和本地信息弹窗） */
export function loaderVisual(type: string): { label: string } {
  const t = type.toLowerCase()
  if (t === 'forge') return { label: 'Forge' }
  if (t === 'neoforge') return { label: 'NeoForge' }
  if (t === 'fabric') return { label: 'Fabric' }
  if (t === 'quilt') return { label: 'Quilt' }
  if (t === 'liteloader') return { label: 'LiteLoader' }
  return { label: '未知' }
}

/**
 * 去除 mod 名称中的版本号等信息，提取纯名称用于百科搜索
 *
 * 例：
 * - "AI-Improvements-1.20-0.5.2.jar" → "AI-Improvements"
 * - "AI-Improvements-1.20-0.5.2" → "AI-Improvements"
 * - "FabricAPI-0.92.2+1.20.4" → "FabricAPI"
 * - "create-1.20.1-6.0.4.jar" → "create"
 *
 * 规则：
 * 1. 先去文件扩展名（.jar / .disabled / .old）
 * 2. 在第一个匹配版本号的位置（如 1.20 / 0.92.2 / 6.0.4）截断
 * 3. 去掉末尾的连字符/下划线/点
 */
export function stripModVersion(name: string): string {
  // 1. 去扩展名
  let s = name.replace(/\.jar(\.disabled|\.old)?$/i, '').replace(/\.(litemod)(\.disabled|\.old)?$/i, '')
  // 2. 在版本号处截断（版本号特征：-<数字>.<数字> 或 +<数字>.<数字> 或 _<数字>.<数字>）
  //    匹配如 -1.20 / -0.92.2 / -6.0.4 / +1.20.4 等
  const m = s.match(/^([^-\s+_]+(?:[-\s+_][^-\s+_]+)*?)[-+_]\d+\.\d+/)
  if (m) {
    return m[1].replace(/[-\s+_]+$/, '')
  }
  return s
}
