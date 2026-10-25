/**
 * 渐变文字工具 - 输出格式编码器
 *
 * 每个格式对应一个独立编码函数，统一在 ENCODERS 里按 id 注册；
 * gradientFormatAdapters 只描述对外展示的元数据（标签 / 示例 / 扩展名）。
 */
import { buildGradientCharacters } from './render'
import { TEXT_FORMATS } from './types'
import type {
  GradientCharacter,
  GradientFormatAdapter,
  GradientFormatId,
  GradientOutputOptions,
  GradientTextDocument,
  TextFormat,
} from './types'

/** 成对包裹式格式标记 */
type FormatPairs = Record<TextFormat, readonly [string, string]>

type Encoder = (cells: GradientCharacter[], options: GradientOutputOptions) => string

const AMPERSAND_PAIRS: FormatPairs = {
  bold: ['&l', ''],
  italic: ['&o', ''],
  underlined: ['&n', ''],
  strikethrough: ['&m', ''],
  obfuscated: ['&k', ''],
}

const MINIMESSAGE_PAIRS: FormatPairs = {
  bold: ['<b>', '</b>'],
  italic: ['<i>', '</i>'],
  underlined: ['<u>', '</u>'],
  strikethrough: ['<st>', '</st>'],
  obfuscated: ['<obf>', '</obf>'],
}

const MINEDOWN_PAIRS: FormatPairs = {
  bold: ['**', '**'],
  italic: ['##', '##'],
  underlined: ['__', '__'],
  strikethrough: ['~~', '~~'],
  obfuscated: ['??', '??'],
}

const CHAT_COLORS_PAIRS: FormatPairs = {
  bold: ['**', '**'],
  italic: ['*', '*'],
  underlined: ['__', '__'],
  strikethrough: ['~~', '~~'],
  obfuscated: ['', ''],
}

const BBCODE_PAIRS: FormatPairs = {
  bold: ['[b]', '[/b]'],
  italic: ['[i]', '[/i]'],
  underlined: ['[u]', '[/u]'],
  strikethrough: ['[s]', '[/s]'],
  obfuscated: ['', ''],
}

const HTML_PAIRS: FormatPairs = {
  bold: ['<b>', '</b>'],
  italic: ['<i>', '</i>'],
  underlined: ['<u>', '</u>'],
  strikethrough: ['<s>', '</s>'],
  obfuscated: ['', ''],
}

/** 原版格式控制字母 */
const LEGACY_LETTERS: Record<TextFormat, string> = {
  bold: 'l',
  italic: 'o',
  underlined: 'n',
  strikethrough: 'm',
  obfuscated: 'k',
}

/** TabooLib gradient 的修饰符字母 */
const TABOOLIB_FLAGS: Record<TextFormat, string> = {
  bold: 'b',
  italic: 'i',
  underlined: 'u',
  strikethrough: 's',
  obfuscated: 'o',
}

/** MOTD 里的控制字符以转义文本形式输出 */
const MOTD_CONTROL = '\\u00A7'

function legacyCodes(formats: TextFormat[], control: string): string {
  return formats.map((format) => `${control}${LEGACY_LETTERS[format]}`).join('')
}

function wrapPairs(text: string, formats: TextFormat[], pairs: FormatPairs): string {
  return formats.reduce((wrapped, format) => {
    const [open, close] = pairs[format]
    return `${open}${wrapped}${close}`
  }, text)
}

function eachCell(cells: GradientCharacter[], render: (cell: GradientCharacter) => string): string {
  return cells.map((cell) => (cell.newline ? '\n' : render(cell))).join('')
}

/** 逐字符编码：颜色前缀 + 格式标记 + 字符 */
function perGlyph(
  cells: GradientCharacter[],
  paint: (cell: GradientCharacter) => string,
  pairs?: FormatPairs,
): string {
  return eachCell(cells, (cell) => {
    const painted = paint(cell)
    return pairs ? wrapPairs(painted, cell.formats, pairs) : painted
  })
}

/** 色带内聚成 run 的编码（MiniMessage / TabooLib / RoseGarden 渐变） */
function byColorRun(
  cells: GradientCharacter[],
  options: GradientOutputOptions,
  render: (text: string, colors: string[], formats: TextFormat[]) => string,
): string {
  const chunks: string[] = []
  let run: GradientCharacter[] = []

  const flush = (): void => {
    if (run.length === 0) return
    const text = run.map((cell) => cell.character).join('')
    const colors = run
      .map((cell) => cell.color)
      .filter((color): color is string => color !== null)
    const stops =
      options.simplifyGradients && colors.length > 1 ? [colors[0], colors[colors.length - 1]] : colors
    chunks.push(render(text, stops, run[0].formats))
    run = []
  }

  for (const cell of cells) {
    if (cell.newline) {
      flush()
      chunks.push('\n')
      continue
    }
    const previous = run[run.length - 1]
    if (previous && (previous.formats.join(',') !== cell.formats.join(',') || cell.color === null)) {
      flush()
    }
    run.push(cell)
  }
  flush()
  return chunks.join('')
}

function quoteSnbt(value: string): string {
  return `'${value.replaceAll('\\', '\\\\').replaceAll("'", "\\'")}'`
}

function csvCell(value: string | boolean): string {
  const text = String(value)
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text
}

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

function encodeVanilla(cells: GradientCharacter[], options: GradientOutputOptions): string {
  const control = options.vanillaCharacter
  return eachCell(cells, (cell) =>
    cell.color
      ? `${control}${cell.color}${legacyCodes(cell.formats, control)}${cell.character}`
      : cell.character,
  )
}

function encodeVanillaCompatible(cells: GradientCharacter[], options: GradientOutputOptions): string {
  const control = options.vanillaCharacter
  return eachCell(cells, (cell) => {
    if (!cell.color) return cell.character
    const digits = cell.color
      .slice(1)
      .split('')
      .map((digit) => `${control}${digit}`)
      .join('')
    return `${control}x${digits}${legacyCodes(cell.formats, control)}${cell.character}`
  })
}

function encodeMotd(cells: GradientCharacter[]): string {
  return eachCell(cells, (cell) => {
    if (!cell.color) return cell.character
    const digits = cell.color
      .slice(1)
      .split('')
      .map((digit) => `${MOTD_CONTROL}${digit}`)
      .join('')
    return `${MOTD_CONTROL}x${digits}${legacyCodes(cell.formats, MOTD_CONTROL)}${cell.character}`
  })
}

function encodeStandard(cells: GradientCharacter[]): string {
  return eachCell(cells, (cell) => (cell.color ? `${cell.color}${cell.character}` : cell.character))
}

function encodeCmi(cells: GradientCharacter[]): string {
  return perGlyph(
    cells,
    (cell) => (cell.color ? `{${cell.color}}${cell.character}` : cell.character),
    AMPERSAND_PAIRS,
  )
}

function encodeMiniMessage(cells: GradientCharacter[]): string {
  return perGlyph(
    cells,
    (cell) => (cell.color ? `<${cell.color}>${cell.character}` : cell.character),
    MINIMESSAGE_PAIRS,
  )
}

function encodeMineDown(cells: GradientCharacter[]): string {
  return perGlyph(
    cells,
    (cell) => (cell.color ? `&${cell.color}&${cell.character}` : cell.character),
    MINEDOWN_PAIRS,
  )
}

function encodeTrChat(cells: GradientCharacter[]): string {
  return perGlyph(
    cells,
    (cell) => (cell.color ? `&{${cell.color}}${cell.character}` : cell.character),
    AMPERSAND_PAIRS,
  )
}

function encodeChatColors(cells: GradientCharacter[]): string {
  return perGlyph(
    cells,
    (cell) => (cell.color ? `[${cell.color}]${cell.character}` : cell.character),
    CHAT_COLORS_PAIRS,
  )
}

function encodeBbcode(cells: GradientCharacter[]): string {
  return perGlyph(
    cells,
    (cell) =>
      cell.color ? `[color=${cell.color}]${cell.character}[/color]` : cell.character,
    BBCODE_PAIRS,
  )
}

function encodeTerraria(cells: GradientCharacter[]): string {
  return perGlyph(cells, (cell) =>
    cell.color ? `[c/${cell.color.slice(1)}:${cell.character}]` : cell.character,
  )
}

function encodeMiniMessageGradient(cells: GradientCharacter[], options: GradientOutputOptions): string {
  return byColorRun(cells, options, (text, colors, formats) =>
    wrapPairs(`<gradient:${colors.join(':')}>${text}</gradient>`, formats, MINIMESSAGE_PAIRS),
  )
}

function encodeTaboolibGradient(cells: GradientCharacter[], options: GradientOutputOptions): string {
  return byColorRun(cells, options, (text, colors, formats) => {
    const flags = formats.map((format) => TABOOLIB_FLAGS[format])
    const suffix = flags.length > 0 ? `;${flags.join(';')}` : ''
    return `[${text}](gradient=${colors.join(',')}${suffix})`
  })
}

function encodeRoseGardenGradient(cells: GradientCharacter[], options: GradientOutputOptions): string {
  return byColorRun(cells, options, (text, colors, formats) =>
    wrapPairs(`<g:${colors.join(':')}>${text}`, formats, MINIMESSAGE_PAIRS),
  )
}

function encodeJson(cells: GradientCharacter[]): string {
  return JSON.stringify(
    cells.map((cell) => {
      const component: Record<string, string | boolean> = { text: cell.character }
      if (cell.color) component.color = cell.color
      for (const format of cell.formats) component[format] = true
      return component
    }),
  )
}

function encodeSnbt(cells: GradientCharacter[]): string {
  const components = cells.map((cell) => {
    const parts = [`text:${quoteSnbt(cell.character)}`]
    if (cell.color) parts.push(`color:${quoteSnbt(cell.color)}`)
    for (const format of cell.formats) parts.push(`${format}:true`)
    return `{${parts.join(',')}}`
  })
  return `[${components.join(',')}]`
}

function encodeHtml(cells: GradientCharacter[]): string {
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

function encodeCsv(cells: GradientCharacter[]): string {
  const header = 'color,char,bold,italic,underlined,strikethrough,obfuscated'
  const rows = cells.map((cell) =>
    [
      cell.color ?? '',
      cell.newline ? '\\n' : cell.character,
      ...TEXT_FORMATS.map((format) => cell.formats.includes(format)),
    ]
      .map(csvCell)
      .join(','),
  )
  return [header, ...rows].join('\n')
}

const ENCODERS: Record<GradientFormatId, Encoder> = {
  vanilla: encodeVanilla,
  'vanilla-compatible': encodeVanillaCompatible,
  standard: encodeStandard,
  cmi: encodeCmi,
  minimessage: encodeMiniMessage,
  'minimessage-gradient': encodeMiniMessageGradient,
  minedown: encodeMineDown,
  snbt: encodeSnbt,
  trchat: encodeTrChat,
  taboolib: encodeTrChat,
  'taboolib-gradient': encodeTaboolibGradient,
  'rosegarden-gradient': encodeRoseGardenGradient,
  'chat-colors': encodeChatColors,
  motd: encodeMotd,
  bbcode: encodeBbcode,
  json: encodeJson,
  html: encodeHtml,
  csv: encodeCsv,
  terraria: encodeTerraria,
}

export const gradientFormatAdapters: readonly GradientFormatAdapter[] = [
  { id: 'vanilla', label: 'Vanilla', sample: '&#RRGGBB', mimeType: 'text/plain', extension: 'txt', supportsVanillaCharacter: true },
  { id: 'vanilla-compatible', label: 'Vanilla compatible', sample: '&x&R&R&G&G&B&B', mimeType: 'text/plain', extension: 'txt', supportsVanillaCharacter: true },
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

/** 按指定格式生成输出 */
export function generateGradientOutput(
  document: GradientTextDocument,
  colors: string[],
  adapterId: GradientFormatId,
  options: GradientOutputOptions,
): string {
  return ENCODERS[adapterId](buildGradientCharacters(document, colors), options)
}
