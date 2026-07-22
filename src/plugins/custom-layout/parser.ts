/**
 * 自定义布局解析器
 *
 * 支持 JSON 和 XML 两种格式，解析后统一转为 LayoutSchema。
 *
 * JSON 格式：
 * ```json
 * {
 *   "title": "我的面板",
 *   "icon": "chart-bar",
 *   "sections": [
 *     { "type": "stat-grid", "columns": 3, "items": [...] },
 *     { "type": "list", "title": "明细", "source": "cache.entries", "fields": [...] },
 *     { "type": "html", "content": "<h1>Hello</h1>", "script": "console.log('hi')", "style": "h1{color:red}", "height": 240 }
 *   ]
 * }
 * ```
 *
 * XML 格式：
 * ```xml
 * <panel title="我的面板" icon="chart-bar">
 *   <stat-grid columns="3">
 *     <item label="总占用" value="{{cache.totalSize}}" format="bytes" />
 *   </stat-grid>
 *   <list title="明细" source="cache.entries">
 *     <field key="name" label="名称" />
 *   </list>
 *   <html height="240">
 *     <content><![CDATA[ <h1>Hello</h1> ]]></content>
 *     <style>h1 { color: red; }</style>
 *     <script>console.log('hi')</script>
 *   </html>
 * </panel>
 * ```
 */

import type { LayoutSchema, LayoutSection, StatItem, ListField, ValueFormat, ParseResult } from './types'

/** 有效的 section 类型 */
const VALID_SECTION_TYPES = new Set(['stat-grid', 'list', 'progress', 'text', 'divider', 'html'])

/** 有效的值格式 */
const VALID_FORMATS = new Set(['bytes', 'number', 'percent', 'text'])

/** 有效的颜色主题 */
const VALID_COLORS = new Set(['primary', 'green', 'yellow', 'red', 'gray'])

/** 有效的文本变体 */
const VALID_VARIANTS = new Set(['default', 'muted', 'warning'])

/**
 * 解析 JSON 布局
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

/**
 * 解析 XML 布局
 *
 * 使用浏览器内置 DOMParser 解析 XML，转为与 JSON 相同的 LayoutSchema。
 *
 * @param content XML 字符串
 * @returns 解析结果（schema 或 error）
 */
export function parseXmlLayout(content: string): ParseResult {
  if (!content.trim()) {
    return { schema: null, error: '内容为空' }
  }

  const parser = new DOMParser()
  const doc = parser.parseFromString(content, 'application/xml')
  const parseError = doc.querySelector('parsererror')
  if (parseError) {
    return { schema: null, error: `XML 解析失败：${parseError.textContent?.trim() ?? '语法错误'}` }
  }

  const panel = doc.documentElement
  if (panel.tagName !== 'panel') {
    return { schema: null, error: `根节点必须是 <panel>，实际为 <${panel.tagName}>` }
  }

  const schema: LayoutSchema = { sections: [] }
  const title = panel.getAttribute('title')
  const icon = panel.getAttribute('icon')
  if (title) schema.title = title
  if (icon) schema.icon = icon

  for (const child of Array.from(panel.children)) {
    const type = child.tagName
    if (!VALID_SECTION_TYPES.has(type)) {
      return { schema: null, error: `未知节点 <${type}>` }
    }

    const section = parseXmlNode(type, child)
    if ('error' in section) {
      return { schema: null, error: section.error }
    }
    schema.sections.push(section.section)
  }

  return { schema, error: null }
}

// ==================== JSON 解析辅助 ====================

interface SectionResult {
  section: LayoutSection
}
interface SectionError {
  error: string
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

// ==================== XML 解析辅助 ====================

/** 解析单个 section（XML） */
function parseXmlNode(type: string, el: Element): SectionResult | SectionError {
  switch (type) {
    case 'stat-grid':
      return parseStatGridXml(el)
    case 'list':
      return parseListXml(el)
    case 'progress':
      return parseProgressXml(el)
    case 'text':
      return parseTextXml(el)
    case 'divider':
      return { section: { type: 'divider' } }
    case 'html':
      return parseHtmlXml(el)
    default:
      return { error: `未知节点 <${type}>` }
  }
}

function parseHtmlXml(el: Element): SectionResult | SectionError {
  // content 必须通过 <content> 子节点提供（避免与 <script>/<style> 文本混淆）
  const contentEl = el.querySelector('content')
  const content = contentEl?.textContent ?? ''
  if (!content.trim()) {
    return { error: 'html 缺少 <content> 子节点（请通过 <content> 提供 HTML 内容）' }
  }
  const section: LayoutSection = { type: 'html', content }
  // script / style 通过子节点 <script> / <style> 提供
  const scriptEl = el.querySelector('script')
  if (scriptEl && scriptEl.textContent) section.script = scriptEl.textContent
  const styleEl = el.querySelector('style')
  if (styleEl && styleEl.textContent) section.style = styleEl.textContent
  const height = el.getAttribute('height')
  if (height) {
    const h = parseInt(height, 10)
    if (h > 0) section.height = h
  }
  return { section }
}

function parseStatGridXml(el: Element): SectionResult | SectionError {
  const items: StatItem[] = []
  for (const itemEl of Array.from(el.children)) {
    if (itemEl.tagName !== 'item') {
      return { error: `stat-grid 内只能包含 <item>，发现 <${itemEl.tagName}>` }
    }
    const label = itemEl.getAttribute('label')
    const value = itemEl.getAttribute('value')
    if (!label) return { error: 'stat-grid item 缺少 label 属性' }
    if (!value) return { error: 'stat-grid item 缺少 value 属性' }
    const item: StatItem = { label, value }
    const format = itemEl.getAttribute('format')
    if (format && VALID_FORMATS.has(format)) item.format = format as ValueFormat
    const color = itemEl.getAttribute('color')
    if (color && VALID_COLORS.has(color)) item.color = color
    items.push(item)
  }

  const section: LayoutSection = { type: 'stat-grid', items }
  const cols = el.getAttribute('columns')
  if (cols) section.columns = parseInt(cols, 10) || undefined
  return { section }
}

function parseListXml(el: Element): SectionResult | SectionError {
  const source = el.getAttribute('source')
  if (!source) return { error: 'list 缺少 source 属性' }

  const fields: ListField[] = []
  for (const fieldEl of Array.from(el.children)) {
    if (fieldEl.tagName !== 'field') {
      return { error: `list 内只能包含 <field>，发现 <${fieldEl.tagName}>` }
    }
    const key = fieldEl.getAttribute('key')
    if (!key) return { error: 'list field 缺少 key 属性' }
    const field: ListField = { key }
    const label = fieldEl.getAttribute('label')
    if (label) field.label = label
    const format = fieldEl.getAttribute('format')
    if (format && VALID_FORMATS.has(format)) field.format = format as ValueFormat
    fields.push(field)
  }

  const section: LayoutSection = { type: 'list', source, fields }
  const title = el.getAttribute('title')
  if (title) section.title = title
  return { section }
}

function parseProgressXml(el: Element): SectionResult | SectionError {
  const value = el.getAttribute('value')
  if (!value) return { error: 'progress 缺少 value 属性' }
  const section: LayoutSection = { type: 'progress', value }
  const label = el.getAttribute('label')
  if (label) section.label = label
  const max = el.getAttribute('max')
  if (max) section.max = max
  const color = el.getAttribute('color')
  if (color && VALID_COLORS.has(color)) section.color = color
  const format = el.getAttribute('format')
  if (format && VALID_FORMATS.has(format)) section.format = format as ValueFormat
  return { section }
}

function parseTextXml(el: Element): SectionResult | SectionError {
  const content = el.textContent?.trim() ?? ''
  if (!content) return { error: 'text 内容为空' }
  const section: LayoutSection = { type: 'text', content }
  const variant = el.getAttribute('variant')
  if (variant && VALID_VARIANTS.has(variant)) {
    section.variant = variant as 'default' | 'muted' | 'warning'
  }
  return { section }
}
