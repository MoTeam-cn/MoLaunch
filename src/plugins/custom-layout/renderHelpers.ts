/**
 * 自定义布局渲染辅助函数
 *
 * 将原本耦合在 CustomLayoutPanel.vue 中的渲染计算逻辑提取为纯函数，
 * 接收 DataContext 作为参数，便于在子组件中复用且可独立测试。
 */
import { resolveValue, formatValue, getListData, type DataContext, type ListEntry } from './datasource'
import type { StatItem, ListField, LayoutSection } from './types'

/** 文本颜色主题映射（stat-grid 文本颜色） */
export const colorClassMap: Record<string, string> = {
  primary: 'text-primary-600',
  green: 'text-green-600',
  yellow: 'text-yellow-600',
  red: 'text-red-600',
  gray: 'text-gray-700',
}

/** 进度条填充颜色映射 */
export const progressBarColorMap: Record<string, string> = {
  primary: 'bg-primary-500',
  green: 'bg-green-500',
  yellow: 'bg-yellow-500',
  red: 'bg-red-500',
  gray: 'bg-gray-500',
}

/** 文本块变体映射 */
export const textVariantMap: Record<string, string> = {
  default: 'text-gray-700',
  muted: 'text-gray-400',
  warning: 'text-yellow-600',
}

/** 解析 stat-grid 项的值 */
export function resolveStatValue(item: StatItem, dataCtx: DataContext): string {
  const raw = resolveValue(item.value, dataCtx)
  return formatValue(raw, item.format)
}

/** 解析进度条当前值 */
export function resolveProgressValue(expr: string, dataCtx: DataContext): number {
  const val = resolveValue(expr, dataCtx)
  return typeof val === 'number' ? val : parseFloat(String(val)) || 0
}

/** 解析进度条最大值（默认 100） */
export function resolveProgressMax(expr: string | undefined, dataCtx: DataContext): number {
  if (!expr) return 100
  const val = resolveValue(expr, dataCtx)
  return typeof val === 'number' ? val : parseFloat(String(val)) || 100
}

/** 进度条百分比（0-100） */
export function progressPercent(
  section: Extract<LayoutSection, { type: 'progress' }>,
  dataCtx: DataContext,
): number {
  const val = resolveProgressValue(section.value, dataCtx)
  const max = resolveProgressMax(section.max, dataCtx)
  if (max <= 0) return 0
  return Math.min(100, Math.max(0, (val / max) * 100))
}

/** 列表数据 */
export function getListEntries(
  section: Extract<LayoutSection, { type: 'list' }>,
  dataCtx: DataContext,
): ListEntry[] {
  return getListData(section.source, dataCtx)
}

/** 格式化列表字段值 */
export function formatFieldValue(entry: ListEntry, field: ListField): string {
  const raw = entry[field.key]
  if (raw === undefined || raw === null) return '-'
  return formatValue(raw, field.format)
}
