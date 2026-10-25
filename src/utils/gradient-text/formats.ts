/**
 * 渐变文字工具 - 输出格式（对应 specs/cleanroom/gradient-text.md §5 §6）
 *
 * 结构：先按规格把每个字符渲染成「颜色前缀 + 字符 + 颜色后缀 + 格式标记」，
 * 再按整段/整文档类型分别聚合。格式表与规格表格一一对应。
 */
import { buildCells } from './cells'
import { TEXT_FORMATS } from './types'
import type {
  GradientCell,
  GradientFormatId,
  GradientFormatInfo,
  GradientOutputOptions,
  GradientTextDocument,
  TextFormat,
} from './types'

/** §6 成对标记表 */
type PairTable = Record<TextFormat, readonly [string, string]>

const MINIMESSAGE_PAIRS: PairTable = {
  bold: ['<b>', '</b>'],
  italic: ['<i>', '</i>'],
  underlined: ['<u>', '</u>'],
  strikethrough: ['<st>', '</st>'],
  obfuscated: ['<obf>', '</obf>'],
}

const MINEDOWN_PAIRS: PairTable = {
  bold: ['**', '**'],
  italic: ['##', '##'],
  underlined: ['__', '__'],
  strikethrough: ['~~', '~~'],
  obfuscated: ['??', '??'],
}

const CHAT_PAIRS: PairTable = {
  bold: ['**', '**'],
  italic: ['*', '*'],
  underlined: ['__', '__'],
  strikethrough: ['~~', '~~'],
  obfuscated: ['', ''],
}

const BBCODE_PAIRS: PairTable = {
  bold: ['[b]', '[/b]'],
  italic: ['[i]', '[/i]'],
  underlined: ['[u]', '[/u]'],
  strikethrough: ['[s]', '[/s]'],
  obfuscated: ['', ''],
}

const HTML_PAIRS: PairTable = {
  bold: ['<b>', '</b>'],
  italic: ['<i>', '</i>'],
  underlined: ['<u>', '</u>'],
  strikethrough: ['<s>', '</s>'],
  obfuscated: ['', ''],
}

/** §6 原版格式字母 */
const LEGACY_LETTERS: Record<TextFormat, string> = {
  bold: 'l',
  italic: 'o',
  underlined: 'n',
  strikethrough: 'm',
  obfuscated: 'k',
}

/** §6 TabooLib gradient 的格式字母 */
const TABOOLIB_LETTERS: Record<TextFormat, string> = {
  bold: 'b',
  italic: 'i',
  underlined: 'u',
  strikethrough: 's',
  obfuscated: 'o',
}

/** MOTD 用转义文本表示控制字符 */
const MOTD_CONTROL = '\\u00A7'

function legacyCodes(formats: TextFormat[], control: string): string {
  return formats.map((format) => `${control}${LEGACY_LETTERS[format]}`).join('')
}

/** §6 成对标记：按 formats 顺序由内向外包裹 */
function wrapPairs(text: string, formats: TextFormat[], pairs: PairTable): string {
  return formats.reduce((wrapped, format) => {
    const [open, close] = pairs[format]
    return `${open}${wrapped}${close}`
  }, text)
}

/** 逐字符输出；换行单元原样输出换行 */
function mapCells(cells: GradientCell[], render: (cell: GradientCell) => string): string {
  return cells.map((cell) => (cell.newline ? '\n' : render(cell))).join('')
}

/** 原版系：颜色前缀里已含格式码 */
function legacyCell(cell: GradientCell, control: string, hex: (color: string) => string): string {
  if (!cell.color) return cell.character
  return `${hex(cell.color)}${legacyCodes(cell.formats, control)}${cell.character}`
}

/** 前缀 + 字符 + 后缀，再套成对格式标记 */
function wrappedCell(cell: GradientCell, prefix: string, suffix: string, pairs: PairTable): string {
  const body = cell.color ? `${prefix}${cell.character}${suffix}` : cell.character
  return wrapPairs(body, cell.formats, pairs)
}

/** §5 整段格式：按同色连续字符合并，格式变化或无色处切段 */
function mapRuns(
  cells: GradientCell[],
  options: GradientOutputOptions,
  render: (text: string, colors: string[], formats: TextFormat[]) => string,
): string {
  const chunks: string[] = []
  let run: GradientCell[] = []

  const flush = (): void => {
    if (run.length === 0) return
    const text = run.map((cell) => cell.character).join('')
    const colors = run.map((cell) => cell.color).filter((color): color is string => color !== null)
    const stops = options.simplifyGradient && colors.length > 1 ? [colors[0], colors[colors.length - 1]] : colors
    chunks.push(render(text, stops, run[0].formats))
    run = []
  }

  for (const cell of cells) {
    if (cell.newline) {
      flush()
      chunks.push('\n')
      continue
    }
    const last = run[run.length - 1]
    if (last && (last.formats.join() !== cell.formats.join() || cell.color === null)) flush()
    run.push(cell)
  }
  flush()
  return chunks.join('')
}

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

function quoteSnbt(value: string): string {
  return `'${value.replaceAll('\\', '\\\\').replaceAll("'", "\\'")}'`
}

function csvField(value: string | boolean): string {
  const text = String(value)
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text
}

/** 整文档渲染签名 */
type Renderer = (cells: GradientCell[], options: GradientOutputOptions) => string

function renderHtml(cells: GradientCell[]): string {
  const lines: string[] = ['']
  for (const cell of cells) {
    if (cell.newline) {
      lines.push('')
      continue
    }
    const glyph = cell.character === ' ' ? '&nbsp;' : escapeHtml(cell.character)
    const painted = cell.color ? `<span style="color: ${cell.color};">${glyph}</span>` : glyph
    lines[lines.length - 1] += wrapPairs(painted, cell.formats, HTML_PAIRS)
  }
  return lines.map((line) => `<p>${line}</p>`).join('')
}

function renderJson(cells: GradientCell[]): string {
  return JSON.stringify(
    cells.map((cell) => {
      const component: Record<string, string | boolean> = { text: cell.character }
      if (cell.color) component.color = cell.color
      for (const format of cell.formats) component[format] = true
      return component
    }),
  )
}

function renderSnbt(cells: GradientCell[]): string {
  const entries = cells.map((cell) => {
    const parts = [`text:${quoteSnbt(cell.character)}`]
    if (cell.color) parts.push(`color:${quoteSnbt(cell.color)}`)
    for (const format of cell.formats) parts.push(`${format}:true`)
    return `{${parts.join(',')}}`
  })
  return `[${entries.join(',')}]`
}

function renderCsv(cells: GradientCell[]): string {
  const header = `color,char,${TEXT_FORMATS.join(',')}`
  const rows = cells.map((cell) =>
    [cell.color ?? '', cell.newline ? '\\n' : cell.character, ...TEXT_FORMATS.map((f) => cell.formats.includes(f))]
      .map(csvField)
      .join(','),
  )
  return [header, ...rows].join('\n')
}

/** 各格式渲染器：键与 §5 表格一致 */
const RENDERERS: Record<GradientFormatId, Renderer> = {
  vanilla: (cells, options) => mapCells(cells, (cell) => legacyCell(cell, options.control, (color) => `${options.control}${color}`)),
  'vanilla-compatible': (cells, options) =>
    mapCells(cells, (cell) =>
      legacyCell(cell, options.control, (color) =>
        `${options.control}x${color
          .slice(1)
          .split('')
          .map((digit) => `${options.control}${digit}`)
          .join('')}`,
      ),
    ),
  motd: (cells) =>
    mapCells(cells, (cell) =>
      legacyCell(cell, MOTD_CONTROL, (color) =>
        `${MOTD_CONTROL}x${color
          .slice(1)
          .split('')
          .map((digit) => `${MOTD_CONTROL}${digit}`)
          .join('')}`,
      ),
    ),
  standard: (cells) => mapCells(cells, (cell) => (cell.color ? `${cell.color}${cell.character}` : cell.character)),
  cmi: (cells) => mapCells(cells, (cell) => legacyCell(cell, '&', (color) => `{${color}}`)),
  trchat: (cells) => mapCells(cells, (cell) => legacyCell(cell, '&', (color) => `&{${color}}`)),
  taboolib: (cells) => mapCells(cells, (cell) => legacyCell(cell, '&', (color) => `&{${color}}`)),
  minimessage: (cells) => mapCells(cells, (cell) => wrappedCell(cell, `<${cell.color}>`, '', MINIMESSAGE_PAIRS)),
  minedown: (cells) => mapCells(cells, (cell) => wrappedCell(cell, `&${cell.color}&`, '', MINEDOWN_PAIRS)),
  'chat-colors': (cells) => mapCells(cells, (cell) => wrappedCell(cell, `[${cell.color}]`, '', CHAT_PAIRS)),
  bbcode: (cells) => mapCells(cells, (cell) => wrappedCell(cell, `[color=${cell.color}]`, '[/color]', BBCODE_PAIRS)),
  terraria: (cells) => mapCells(cells, (cell) => (cell.color ? `[c/${cell.color.slice(1)}:${cell.character}]` : cell.character)),
  'minimessage-gradient': (cells, options) =>
    mapRuns(cells, options, (text, colors, formats) =>
      wrapPairs(`<gradient:${colors.join(':')}>${text}</gradient>`, formats, MINIMESSAGE_PAIRS),
    ),
  'rosegarden-gradient': (cells, options) =>
    mapRuns(cells, options, (text, colors, formats) =>
      wrapPairs(`<g:${colors.join(':')}>${text}`, formats, MINIMESSAGE_PAIRS),
    ),
  'taboolib-gradient': (cells, options) =>
    mapRuns(cells, options, (text, colors, formats) => {
      const flags = formats.map((format) => TABOOLIB_LETTERS[format])
      const suffix = flags.length > 0 ? `;${flags.join(';')}` : ''
      return `[${text}](gradient=${colors.join(',')}${suffix})`
    }),
  html: (cells) => renderHtml(cells),
  json: (cells) => renderJson(cells),
  snbt: (cells) => renderSnbt(cells),
  csv: (cells) => renderCsv(cells),
}

/** §5 元数据（界面展示用） */
export const gradientFormatAdapters: readonly GradientFormatInfo[] = [
  { id: 'vanilla', label: 'Vanilla', sample: '&#RRGGBB', mimeType: 'text/plain', extension: 'txt', usesControl: true },
  { id: 'vanilla-compatible', label: 'Vanilla compatible', sample: '&x&R&R&G&G&B&B', mimeType: 'text/plain', extension: 'txt', usesControl: true },
  { id: 'standard', label: 'Standard HEX', sample: '#RRGGBB', mimeType: 'text/plain', extension: 'txt' },
  { id: 'cmi', label: 'CMI', sample: '{#RRGGBB}', mimeType: 'text/plain', extension: 'txt' },
  { id: 'minimessage', label: 'MiniMessage', sample: '<#RRGGBB>', mimeType: 'text/plain', extension: 'txt' },
  { id: 'minimessage-gradient', label: 'MiniMessage gradient', sample: '<gradient:#RRGGBB:#RRGGBB>', mimeType: 'text/plain', extension: 'txt', supportsSimplify: true },
  { id: 'minedown', label: 'MineDown', sample: '&#RRGGBB&', mimeType: 'text/plain', extension: 'txt' },
  { id: 'snbt', label: 'Stringified NBT', sample: "{text:'T',color:'#RRGGBB'}", mimeType: 'application/json', extension: 'snbt' },
  { id: 'trchat', label: 'TrChat', sample: '&{#RRGGBB}', mimeType: 'text/plain', extension: 'txt' },
  { id: 'taboolib', label: 'TabooLib', sample: '&{#RRGGBB}', mimeType: 'text/plain', extension: 'txt' },
  { id: 'taboolib-gradient', label: 'TabooLib gradient', sample: '[Text](gradient=#RRGGBB,#RRGGBB)', mimeType: 'text/plain', extension: 'txt', supportsSimplify: true },
  { id: 'rosegarden-gradient', label: 'RoseGarden gradient', sample: '<g:#RRGGBB:#RRGGBB>Text', mimeType: 'text/plain', extension: 'txt', supportsSimplify: true },
  { id: 'chat-colors', label: 'Chat Colors', sample: '[#RRGGBB]', mimeType: 'text/plain', extension: 'txt' },
  { id: 'motd', label: 'MOTD', sample: '\\u00A7x', mimeType: 'text/plain', extension: 'txt' },
  { id: 'bbcode', label: 'BBCode', sample: '[color=#RRGGBB]Text[/color]', mimeType: 'text/plain', extension: 'txt' },
  { id: 'json', label: 'JSON text component', sample: '{"text":"T","color":"#RRGGBB"}', mimeType: 'application/json', extension: 'json' },
  { id: 'html', label: 'HTML', sample: '<span style="color: #RRGGBB">Text</span>', mimeType: 'text/html', extension: 'html' },
  { id: 'csv', label: 'CSV', sample: '#RRGGBB,T', mimeType: 'text/csv', extension: 'csv' },
  { id: 'terraria', label: 'Terraria', sample: '[c/RRGGBB:T]', mimeType: 'text/plain', extension: 'txt' },
]

/** 生成指定格式的输出 */
export function generateGradientOutput(
  document: GradientTextDocument,
  colors: string[],
  formatId: GradientFormatId,
  options: GradientOutputOptions,
): string {
  return RENDERERS[formatId](buildCells(document, colors), options)
}
