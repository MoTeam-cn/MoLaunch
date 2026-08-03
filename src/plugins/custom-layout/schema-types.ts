import type { LayoutSection } from './types'

/** 有效的 section 类型 */
export const VALID_SECTION_TYPES = new Set(['stat-grid', 'list', 'progress', 'text', 'divider', 'html'])

/** 有效的值格式 */
export const VALID_FORMATS = new Set(['bytes', 'number', 'percent', 'text', 'datetime'])

/** 有效的颜色主题 */
export const VALID_COLORS = new Set(['primary', 'green', 'yellow', 'red', 'gray'])

/** 有效的文本变体 */
export const VALID_VARIANTS = new Set(['default', 'muted', 'warning'])

export interface SectionResult {
  section: LayoutSection
}
export interface SectionError {
  error: string
}