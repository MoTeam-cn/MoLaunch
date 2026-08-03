import type { LayoutSchema, ListField, LayoutSection, ParseResult, StatItem, ValueFormat } from './types'
import { VALID_COLORS, VALID_FORMATS, VALID_SECTION_TYPES, VALID_VARIANTS, type SectionError, type SectionResult } from './schema-types'

/**
 * 解析 XML 布局
 *
 * 使用浏览器内置 DOMParser 解析 XML，转为与 JSON 相同的 LayoutSchema。
 * 格式示例：<panel title="我的面板" icon="chart-bar">
 *   <stat-grid columns="3"><item label="总占用" value="{{cache.totalSize}}" format="bytes" /></stat-grid>
 *   <list title="明细" source="cache.entries"><field key="name" label="名称" /></list>
 *   <html height="240"><content><![CDATA[ <h1>Hello</h1> ]]></content></html>
 * </panel>
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