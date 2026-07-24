/**
 * 自定义布局数据源
 *
 * 提供布局渲染所需的实时数据，通过 pluginSdk 获取。
 *
 * 支持的数据源路径：
 * - cache.totalSize：缓存总大小（字节）
 * - cache.totalFiles：缓存总文件数
 * - cache.cleanableSize：可自动清理大小（字节）
 * - cache.entries：缓存条目列表（含 name / fileCount / totalSize / ttlHours / path / category / subDir）
 * - system.totalMemory：系统总内存（字节）
 * - system.usedMemory：已用内存（字节）
 * - system.availableMemory：可用内存（字节）
 * - system.usagePercent：内存使用率（0-100）
 * - versions.count：已安装版本数
 * - versions.list：已安装版本列表（含 id / version_type / logo）
 * - history.count：启动历史记录数
 * - history.recent：最近启动记录（含 version_id / username / launch_time / pid / exit_code）
 */

import { pluginSdk } from '@/plugins/sdk'
import { formatBytes } from '@/utils/format'

/** 数据源上下文（扁平化的键值对，供 {{key}} 插值） */
export interface DataContext {
  [key: string]: unknown
}

/** 列表数据条目（任意字段的对象） */
export type ListEntry = Record<string, unknown>

/**
 * 加载所有数据源，返回扁平化的上下文对象
 *
 * 并行获取所有数据源，失败的数据源填入 undefined（不中断其他数据源）。
 */
export async function loadDataContext(): Promise<DataContext> {
  const [cacheResult, memResult, versionsResult, historyResult] = await Promise.allSettled([
    pluginSdk.getCacheStats(),
    pluginSdk.getSystemMemory(),
    pluginSdk.listInstalledVersionsWithType(),
    pluginSdk.listLaunchHistory(),
  ])

  const ctx: DataContext = {}

  // 缓存数据
  if (cacheResult.status === 'fulfilled') {
    const cache = cacheResult.value
    const allEntries = [...cache.cache, ...cache.cacheTemp, ...cache.cacheApp]
    ctx['cache.totalSize'] = allEntries.reduce((s, e) => s + e.totalSize, 0)
    ctx['cache.totalFiles'] = allEntries.reduce((s, e) => s + e.fileCount, 0)
    ctx['cache.cleanableSize'] = allEntries
      .filter((e) => e.ttlHours !== null)
      .reduce((s, e) => s + e.totalSize, 0)
    ctx['cache.entries'] = allEntries
  }

  // 系统内存
  if (memResult.status === 'fulfilled') {
    const mem = memResult.value
    ctx['system.totalMemory'] = mem.total
    ctx['system.usedMemory'] = mem.used
    ctx['system.availableMemory'] = mem.available
    ctx['system.usagePercent'] = mem.usage_percent
  }

  // 版本列表
  if (versionsResult.status === 'fulfilled') {
    const versions = versionsResult.value
    ctx['versions.count'] = versions.length
    ctx['versions.list'] = versions
  }

  // 启动历史
  if (historyResult.status === 'fulfilled') {
    const history = historyResult.value
    ctx['history.count'] = history.length
    ctx['history.recent'] = history
  }

  return ctx
}

/**
 * 从数据源路径获取列表数据
 *
 * @param source 数据源路径（如 "cache.entries" / "versions.list" / "history.recent"）
 * @param ctx 数据上下文
 * @returns 列表条目数组（找不到时返回空数组）
 */
export function getListData(source: string, ctx: DataContext): ListEntry[] {
  const data = ctx[source]
  if (!Array.isArray(data)) return []
  return data as ListEntry[]
}

/**
 * 解析值表达式中的 {{dataSource.field}} 插值
 *
 * 例如 "{{cache.totalSize}}" → ctx['cache.totalSize'] 的值
 * 支持多个插值： "{{system.usedMemory}} / {{system.totalMemory}}"
 *
 * @param expr 值表达式
 * @param ctx 数据上下文
 * @returns 解析后的值（数值或字符串）
 */
export function resolveValue(expr: string, ctx: DataContext): string | number {
  // 匹配 {{xxx}} 模式
  const matches = expr.match(/\{\{([^}]+)\}\}/g)
  if (!matches) return expr

  let result = expr
  for (const match of matches) {
    const key = match.slice(2, -2).trim()
    const val = ctx[key]
    if (val === undefined || val === null) {
      result = result.replace(match, '-')
    } else if (typeof val === 'number') {
      result = result.replace(match, String(val))
    } else {
      result = result.replace(match, String(val))
    }
  }

  // 如果整个表达式就是单个 {{xxx}}，返回原始值（保留数值类型）
  if (matches.length === 1 && expr.trim() === matches[0]) {
    const key = matches[0].slice(2, -2).trim()
    const val = ctx[key]
    if (typeof val === 'number') return val
    if (typeof val === 'string') return val
    return result
  }

  return result
}

/**
 * 格式化值
 *
 * @param value 原始值（数值或字符串）
 * @param format 格式化方式
 */
export function formatValue(value: string | number, format?: string): string {
  const num = typeof value === 'number' ? value : parseFloat(String(value))

  switch (format) {
    case 'bytes':
      if (isNaN(num)) return String(value)
      return formatBytes(num)
    case 'number':
      if (isNaN(num)) return String(value)
      return num.toLocaleString()
    case 'percent':
      if (isNaN(num)) return String(value)
      return `${num.toFixed(1)}%`
    case 'text':
    default:
      return String(value)
  }
}
