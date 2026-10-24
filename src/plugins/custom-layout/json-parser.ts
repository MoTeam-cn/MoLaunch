import type { ButtonAction, LayoutSchema, ListField, LayoutSection, ParseResult, StatItem, ValueFormat } from './types'
import {
  ALLOWED_NAV_PATHS,
  VALID_ACTION_TYPES,
  VALID_COLORS,
  VALID_FORMATS,
  VALID_SECTION_TYPES,
  VALID_VARIANTS,
  type SectionError,
  type SectionResult,
} from './schema-types'

/**
 * 解析 JSON 布局
 *
 * 格式示例：{ "title": "我的面板", "icon": "chart-bar", "sections": [
 *   { "type": "stat-grid", "columns": 3, "items": [...] },
 *   { "type": "list", "title": "明细", "source": "cache.entries", "fields": [...] },
 *   { "type": "html", "content": "<h1>Hello</h1>", "script": "console.log('hi')", "style": "h1{color:red}", "height": 240 } ] }
 * 合法 section 类型见 VALID_SECTION_TYPES，值格式见 VALID_FORMATS。
 *
 * @param content JSON 字符串
 * @returns 解析结果（schema 或 error）
 */
export function parseJsonLayout(content: string): ParseResult {
  if (!content.trim()) {
    return { schema: null, error: '内容为空' }
  }

  let obj: unknown
  try {
    obj = JSON.parse(content)
  } catch (e) {
    return { schema: null, error: `JSON 解析失败：${e}` }
  }

  const root = obj as Record<string, unknown>
  if (!root || typeof root !== 'object') {
    return { schema: null, error: '根节点必须是对象' }
  }

  const sectionsRaw = root.sections
  if (!Array.isArray(sectionsRaw)) {
    return { schema: null, error: 'sections 字段必须是数组' }
  }

  const sections: LayoutSection[] = []
  for (let i = 0; i < sectionsRaw.length; i++) {
    const s = sectionsRaw[i] as Record<string, unknown>
    const type = String(s?.type ?? '')
    if (!VALID_SECTION_TYPES.has(type)) {
      return { schema: null, error: `sections[${i}] 类型无效：${type}` }
    }

    const section = parseSection(type, s)
    if ('error' in section) {
      return { schema: null, error: `sections[${i}] ${section.error}` }
    }
    sections.push(section.section)
  }

  const schema: LayoutSchema = { sections }
  if (typeof root.title === 'string') schema.title = root.title
  if (typeof root.icon === 'string') schema.icon = root.icon

  return { schema, error: null }
}

/** 解析单个 section（JSON） */
function parseSection(type: string, s: Record<string, unknown>): SectionResult | SectionError {
  switch (type) {
    case 'stat-grid':
      return parseStatGridJson(s)
    case 'list':
      return parseListJson(s)
    case 'progress':
      return parseProgressJson(s)
    case 'text':
      return parseTextJson(s)
    case 'divider':
      return { section: { type: 'divider' } }
    case 'button':
      return parseButtonJson(s)
    case 'html':
      return parseHtmlJson(s)
    default:
      return { error: `未知类型：${type}` }
  }
}

function parseHtmlJson(s: Record<string, unknown>): SectionResult | SectionError {
  if (typeof s.content !== 'string') {
    return { error: 'html.content 必须是字符串' }
  }
  const section: LayoutSection = { type: 'html', content: s.content }
  if (typeof s.script === 'string') section.script = s.script
  if (typeof s.style === 'string') section.style = s.style
  if (typeof s.height === 'number' && s.height > 0) section.height = s.height
  return { section }
}

function parseStatGridJson(s: Record<string, unknown>): SectionResult | SectionError {
  const itemsRaw = s.items
  if (!Array.isArray(itemsRaw)) {
    return { error: 'stat-grid.items 必须是数组' }
  }

  const items: StatItem[] = []
  for (let i = 0; i < itemsRaw.length; i++) {
    const item = itemsRaw[i] as Record<string, unknown>
    if (typeof item.label !== 'string') {
      return { error: `stat-grid.items[${i}].label 必须是字符串` }
    }
    if (typeof item.value !== 'string') {
      return { error: `stat-grid.items[${i}].value 必须是字符串` }
    }
    const statItem: StatItem = { label: item.label, value: item.value }
    if (typeof item.format === 'string' && VALID_FORMATS.has(item.format)) {
      statItem.format = item.format as ValueFormat
    }
    if (typeof item.color === 'string' && VALID_COLORS.has(item.color)) {
      statItem.color = item.color
    }
    items.push(statItem)
  }

  const section: LayoutSection = { type: 'stat-grid', items }
  if (typeof s.columns === 'number') {
    section.columns = s.columns
  }
  return { section }
}

function parseListJson(s: Record<string, unknown>): SectionResult | SectionError {
  if (typeof s.source !== 'string') {
    return { error: 'list.source 必须是字符串' }
  }
  const fieldsRaw = s.fields
  if (!Array.isArray(fieldsRaw)) {
    return { error: 'list.fields 必须是数组' }
  }

  const fields: ListField[] = []
  for (let i = 0; i < fieldsRaw.length; i++) {
    const f = fieldsRaw[i] as Record<string, unknown>
    if (typeof f.key !== 'string') {
      return { error: `list.fields[${i}].key 必须是字符串` }
    }
    const field: ListField = { key: f.key }
    if (typeof f.label === 'string') field.label = f.label
    if (typeof f.format === 'string' && VALID_FORMATS.has(f.format)) {
      field.format = f.format as ValueFormat
    }
    fields.push(field)
  }

  const section: LayoutSection = { type: 'list', source: s.source, fields }
  if (typeof s.title === 'string') section.title = s.title
  return { section }
}

function parseProgressJson(s: Record<string, unknown>): SectionResult | SectionError {
  if (typeof s.value !== 'string') {
    return { error: 'progress.value 必须是字符串' }
  }
  const section: LayoutSection = { type: 'progress', value: s.value }
  if (typeof s.label === 'string') section.label = s.label
  if (typeof s.max === 'string') section.max = s.max
  if (typeof s.color === 'string' && VALID_COLORS.has(s.color)) {
    section.color = s.color
  }
  if (typeof s.format === 'string' && VALID_FORMATS.has(s.format)) {
    section.format = s.format as ValueFormat
  }
  return { section }
}

function parseTextJson(s: Record<string, unknown>): SectionResult | SectionError {
  if (typeof s.content !== 'string') {
    return { error: 'text.content 必须是字符串' }
  }
  const section: LayoutSection = { type: 'text', content: s.content }
  if (typeof s.variant === 'string' && VALID_VARIANTS.has(s.variant)) {
    section.variant = s.variant as 'default' | 'muted' | 'warning'
  }
  return { section }
}

/** 解析 button section（JSON） */
function parseButtonJson(s: Record<string, unknown>): SectionResult | SectionError {
  const label = typeof s.label === 'string' ? s.label.trim() : ''
  if (!label) {
    return { error: 'button 缺少 label' }
  }
  const rawAction = s.action as Record<string, unknown> | undefined
  if (!rawAction || typeof rawAction !== 'object') {
    return { error: 'button 缺少 action' }
  }
  const parsed = parseActionJson(rawAction)
  if ('error' in parsed) {
    return parsed
  }
  const section: LayoutSection = { type: 'button', label, action: parsed.action }
  const variant = String(s.variant ?? '')
  if (variant === 'primary' || variant === 'outline') {
    section.variant = variant
  }
  return { section }
}

/** 解析按钮动作（JSON），动作参数非法时返回错误 */
function parseActionJson(raw: Record<string, unknown>): { action: ButtonAction } | SectionError {
  const type = String(raw.type ?? '')
  if (!VALID_ACTION_TYPES.has(type)) {
    return { error: `button 动作类型无效：${type}` }
  }
  switch (type) {
    case 'launch': {
      const version = typeof raw.version === 'string' && raw.version.trim() ? raw.version.trim() : undefined
      return { action: version ? { type: 'launch', version } : { type: 'launch' } }
    }
    case 'open_url': {
      const url = typeof raw.url === 'string' ? raw.url.trim() : ''
      if (!/^https?:\/\//i.test(url)) {
        return { error: 'button 的 url 必须是 http(s) 地址' }
      }
      return { action: { type: 'open_url', url } }
    }
    case 'navigate': {
      const path = typeof raw.path === 'string' ? raw.path.trim() : ''
      if (!ALLOWED_NAV_PATHS.has(path)) {
        return { error: `button 的 path 不在允许列表内：${path}` }
      }
      return { action: { type: 'navigate', path } }
    }
    case 'copy': {
      const text = typeof raw.text === 'string' ? raw.text : ''
      if (!text) {
        return { error: 'button 的 copy 动作缺少 text' }
      }
      return { action: { type: 'copy', text } }
    }
    default:
      return { error: `button 动作类型无效：${type}` }
  }
}
