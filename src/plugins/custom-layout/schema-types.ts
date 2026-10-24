import type { LayoutSection } from './types'

/** 有效的 section 类型 */
export const VALID_SECTION_TYPES = new Set([
  'stat-grid',
  'list',
  'progress',
  'text',
  'divider',
  'html',
  'button',
])

/** 有效的值格式 */
export const VALID_FORMATS = new Set(['bytes', 'number', 'percent', 'text', 'datetime'])

/** 有效的颜色主题 */
export const VALID_COLORS = new Set(['primary', 'green', 'yellow', 'red', 'gray'])

/** 有效的文本变体 */
export const VALID_VARIANTS = new Set(['default', 'muted', 'warning'])

/** 有效的按钮动作类型 */
export const VALID_ACTION_TYPES = new Set(['launch', 'open_url', 'navigate', 'copy'])

/** 动作允许跳转的页面白名单 */
export const ALLOWED_NAV_PATHS = new Set([
  '/apps',
  '/apps/versions',
  '/apps/downloads',
  '/apps/tools',
  '/apps/online',
  '/apps/settings',
])

export interface SectionResult {
  section: LayoutSection
}
export interface SectionError {
  error: string
}