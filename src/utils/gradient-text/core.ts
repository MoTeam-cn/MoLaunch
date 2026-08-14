/**
 * 渐变文字工具 - 核心算法
 *
 * 颜色插值 / 字符级展开 / Minecraft 阴影色计算。
 * 阴影色按 MC 原版规则：HSL 亮度取 RGB 最大值 1/4。
 */

import type {
  GradientCharacter,
  GradientColor,
  GradientFormatId,
  GradientOutputOptions,
  GradientTextDocument,
  GradientTextRun,
  TextFormat,
} from './types'
import { TEXT_FORMATS } from './types'

export const DEFAULT_GRADIENT_COLORS = ['#A855F7', '#22C55E']

export const DEFAULT_GRADIENT_DOCUMENT: GradientTextDocument = {
  lines: [[{ text: 'MoLaunch', formats: [] }]],
}

export function cloneGradientDocument(document: GradientTextDocument): GradientTextDocument {
  return {
    lines: document.lines.map((line) =>
      line.map((run) => ({ text: run.text, formats: [...run.formats] })),
    ),
  }
}

export function createDocumentFromPlainText(text: string): GradientTextDocument {
  return {
    lines: text.split('\n').map((line) => [{ text: line, formats: [] }]),
  }
}

export function plainTextFromDocument(document: GradientTextDocument): string {
  return document.lines.map((line) => line.map((run) => run.text).join('')).join('\n')
}

export function normalizeHexColor(value: string): string | null {
  const match = value.trim().match(/^#?([0-9a-f]{3}|[0-9a-f]{6})$/i)
  if (!match) return null
  const expanded =
    match[1].length === 3
      ? match[1]
          .split('')
          .map((part) => `${part}${part}`)
          .join('')
      : match[1]
  return `#${expanded.toUpperCase()}`
}

export function parseGradientColors(value: string): string[] {
  const colors: string[] = []
  for (const match of value.matchAll(/#(?:[0-9a-f]{3}|[0-9a-f]{6})\b/gi)) {
    const color = normalizeHexColor(match[0])
    if (color) colors.push(color)
  }
  for (const match of value.matchAll(/rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})/gi)) {
    const components = match.slice(1, 4).map(Number)
    if (components.every((component) => component >= 0 && component <= 255)) {
      colors.push(colorToHex({ red: components[0], green: components[1], blue: components[2] }))
    }
  }
  return [...new Set(colors)]
}

export function interpolateGradient(colors: string[], count: number): string[] {
  const normalized = colors
    .map(normalizeHexColor)
    .filter((color): color is string => color !== null)
  if (count <= 0 || normalized.length === 0) return []
  if (normalized.length === 1 || count === 1) {
    return Array.from({ length: count }, () => normalized[0])
  }

  const source = normalized.map(hexToColor)
  return Array.from({ length: count }, (_, index) => {
    const position = index / (count - 1)
    const scaled = position * (source.length - 1)
    const startIndex = Math.floor(scaled)
    const endIndex = Math.min(source.length - 1, startIndex + 1)
    const progress = scaled - startIndex
    const start = source[startIndex]
    const end = source[endIndex]
    return colorToHex({
      red: Math.round(start.red + (end.red - start.red) * progress),
      green: Math.round(start.green + (end.green - start.green) * progress),
      blue: Math.round(start.blue + (end.blue - start.blue) * progress),
    })
  })
}

export function getMinecraftTextShadow(color: string | null): string {
  const normalized = color ? normalizeHexColor(color) : null
  if (!normalized) return '#000000'

  const { red, green, blue } = hexToColor(normalized)
  const redN = red / 255
  const greenN = green / 255
  const blueN = blue / 255
  const maximum = Math.max(redN, greenN, blueN)
  const minimum = Math.min(redN, greenN, blueN)
  const delta = maximum - minimum
  const saturation = maximum === 0 ? 0 : delta / maximum
  let hue = 0

  if (delta !== 0) {
    if (maximum === redN) {
      hue = ((greenN - blueN) / delta) % 6
    } else if (maximum === greenN) {
      hue = (blueN - redN) / delta + 2
    } else {
      hue = (redN - greenN) / delta + 4
    }
    hue = ((hue * 60 + 360) % 360) / 360
  }

  const lightness = maximum / 4
  const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation
  const segment = hue * 6
  const secondary = chroma * (1 - Math.abs((segment % 2) - 1))
  const match = lightness - chroma / 2
  const [shadowRed, shadowGreen, shadowBlue] =
    segment < 1
      ? [chroma, secondary, 0]
      : segment < 2
        ? [secondary, chroma, 0]
        : segment < 3
          ? [0, chroma, secondary]
          : segment < 4
            ? [0, secondary, chroma]
            : segment < 5
              ? [secondary, 0, chroma]
              : [chroma, 0, secondary]

  return colorToHex({
    red: Math.round((shadowRed + match) * 255),
    green: Math.round((shadowGreen + match) * 255),
    blue: Math.round((shadowBlue + match) * 255),
  })
}

export function buildGradientCharacters(
  document: GradientTextDocument,
  colors: string[],
): GradientCharacter[] {
  const normalized = normalizeGradientDocument(document)
  const uncoloredCharacterCount = normalized.lines.reduce(
    (total, line) =>
      total +
      line.reduce(
        (lineTotal, run) =>
          lineTotal + Array.from(run.text).filter((character) => !/\s/u.test(character)).length,
        0,
      ),
    0,
  )
  const gradient = interpolateGradient(colors, uncoloredCharacterCount)
  let colorIndex = 0
  const characters: GradientCharacter[] = []

  normalized.lines.forEach((line, lineIndex) => {
    line.forEach((run) => {
      for (const character of Array.from(run.text)) {
        const color = /\s/u.test(character) ? null : (gradient[colorIndex++] ?? null)
        characters.push({ character, color, formats: run.formats })
      }
    })
    if (lineIndex < normalized.lines.length - 1) {
      characters.push({ character: '\n', color: null, formats: [], newline: true })
    }
  })

  return characters
}

export function normalizeGradientDocument(value: GradientTextDocument): GradientTextDocument {
  const lines = Array.isArray(value?.lines) ? value.lines : []
  const normalizedLines = lines.map((line) => {
    if (!Array.isArray(line)) return []
    return line
      .filter((run): run is GradientTextRun => typeof run?.text === 'string')
      .map((run) => ({
        text: run.text,
        formats: TEXT_FORMATS.filter(
          (format) => Array.isArray(run.formats) && run.formats.includes(format),
        ),
      }))
  })
  return { lines: normalizedLines.length ? normalizedLines : [[{ text: '', formats: [] }]] }
}

export function generateGradientOutput(
  document: GradientTextDocument,
  colors: string[],
  adapterId: GradientFormatId,
  options: GradientOutputOptions,
): string {
  const characters = buildGradientCharacters(document, colors)
  return formatByAdapter(characters, adapterId, options)
}

type FormatMap = Record<TextFormat, readonly [string, string]>

const minimessageFormats: FormatMap = {
  bold: ['<b>', '</b>'],
  italic: ['<i>', '</i>'],
  underlined: ['<u>', '</u>'],
  strikethrough: ['<st>', '</st>'],
  obfuscated: ['<obf>', '</obf>'],
}

const ampersandFormats: FormatMap = {
  bold: ['&l', ''],
  italic: ['&o', ''],
  underlined: ['&n', ''],
  strikethrough: ['&m', ''],
  obfuscated: ['&k', ''],
}

const minedownFormats: FormatMap = {
  bold: ['**', '**'],
  italic: ['##', '##'],
  underlined: ['__', '__'],
  strikethrough: ['~~', '~~'],
  obfuscated: ['??', '??'],
}

const chatColorsFormats: FormatMap = {
  bold: ['**', '**'],
  italic: ['*', '*'],
  underlined: ['__', '__'],
  strikethrough: ['~~', '~~'],
  obfuscated: ['', ''],
}

const bbcodeFormats: FormatMap = {
  bold: ['[b]', '[/b]'],
  italic: ['[i]', '[/i]'],
  underlined: ['[u]', '[/u]'],
  strikethrough: ['[s]', '[/s]'],
  obfuscated: ['', ''],
}

const htmlFormats: FormatMap = {
  bold: ['<b>', '</b>'],
  italic: ['<i>', '</i>'],
  underlined: ['<u>', '</u>'],
  strikethrough: ['<s>', '</s>'],
  obfuscated: ['', ''],
}

const taboolibFormats: Record<TextFormat, string> = {
  bold: 'b',
  italic: 'i',
  underlined: 'u',
  strikethrough: 's',
  obfuscated: 'o',
}

function formatByAdapter(
  characters: GradientCharacter[],
  adapterId: GradientFormatId,
  options: GradientOutputOptions,
): string {
  switch (adapterId) {
    case 'vanilla':
      return formatVanillaCharacters(characters, options.vanillaCharacter)
    case 'vanilla-compatible':
      return formatVanillaCompatibleCharacters(characters, options.vanillaCharacter)
    case 'standard':
      return formatEachCharacter(characters, (character) =>
        character.color ? `${character.color}${character.character}` : character.character,
      )
    case 'cmi':
      return formatEachCharacter(
        characters,
        (character) =>
          character.color ? `{${character.color}}${character.character}` : character.character,
        ampersandFormats,
      )
    case 'minimessage':
      return formatEachCharacter(
        characters,
        (character) =>
          character.color ? `<${character.color}>${character.character}` : character.character,
        minimessageFormats,
      )
    case 'minimessage-gradient':
      return formatGradientRuns(characters, options, (text, runColors, formats) =>
        wrapFormats(`<gradient:${runColors.join(':')}>${text}</gradient>`, formats, minimessageFormats),
      )
    case 'minedown':
      return formatEachCharacter(
        characters,
        (character) =>
          character.color ? `&${character.color}&${character.character}` : character.character,
        minedownFormats,
      )
    case 'snbt':
      return toSnbt(characters)
    case 'trchat':
    case 'taboolib':
      return formatEachCharacter(
        characters,
        (character) =>
          character.color ? `&{${character.color}}${character.character}` : character.character,
        ampersandFormats,
      )
    case 'taboolib-gradient':
      return formatGradientRuns(characters, options, (text, runColors, formats) => {
        const modifiers = formats.map((format) => taboolibFormats[format]).filter(Boolean)
        return `[${text}](gradient=${runColors.join(',')}${modifiers.length ? `;${modifiers.join(';')}` : ''})`
      })
    case 'rosegarden-gradient':
      return formatGradientRuns(characters, options, (text, runColors, formats) =>
        wrapFormats(`<g:${runColors.join(':')}>${text}`, formats, minimessageFormats),
      )
    case 'chat-colors':
      return formatEachCharacter(
        characters,
        (character) =>
          character.color ? `[${character.color}]${character.character}` : character.character,
        chatColorsFormats,
      )
    case 'motd':
      return formatMotdCharacters(characters)
    case 'bbcode':
      return formatEachCharacter(
        characters,
        (character) =>
          character.color
            ? `[color=${character.color}]${character.character}[/color]`
            : character.character,
        bbcodeFormats,
      )
    case 'json':
      return JSON.stringify(toTextComponents(characters))
    case 'html':
      return toHtml(characters)
    case 'csv':
      return toCsv(characters)
    case 'terraria':
      return formatEachCharacter(characters, (character) =>
        character.color
          ? `[c/${character.color.slice(1)}:${character.character}]`
          : character.character,
      )
  }
}

function formatEachCharacter(
  characters: GradientCharacter[],
  formatter: (character: GradientCharacter) => string,
  formatMap?: FormatMap,
): string {
  return characters
    .map((character) => {
      if (character.newline) return '\n'
      const formatted = formatter(character)
      return formatMap ? wrapFormats(formatted, character.formats, formatMap) : formatted
    })
    .join('')
}

function formatLegacyCharacters(
  characters: GradientCharacter[],
  formatter: (character: GradientCharacter) => string,
): string {
  return characters
    .map((character) => (character.newline ? '\n' : formatter(character)))
    .join('')
}

function formatVanillaCharacters(
  characters: GradientCharacter[],
  characterCode: '&' | '§',
): string {
  return formatLegacyCharacters(characters, (character) => {
    if (!character.color) return character.character
    return `${characterCode}${character.color}${legacyFormatCodes(character.formats, characterCode)}${character.character}`
  })
}

function formatVanillaCompatibleCharacters(
  characters: GradientCharacter[],
  characterCode: '&' | '§',
): string {
  return formatLegacyCharacters(characters, (character) => {
    if (!character.color) return character.character
    const hexadecimal = character.color
      .slice(1)
      .split('')
      .map((part) => `${characterCode}${part}`)
      .join('')
    return `${characterCode}x${hexadecimal}${legacyFormatCodes(character.formats, characterCode)}${character.character}`
  })
}

function formatMotdCharacters(characters: GradientCharacter[]): string {
  return formatLegacyCharacters(characters, (character) => {
    if (!character.color) return character.character
    const hexadecimal = character.color
      .slice(1)
      .split('')
      .map((part) => `\\u00A7${part}`)
      .join('')
    return `\\u00A7x${hexadecimal}${legacyFormatCodes(character.formats, '\\u00A7')}${character.character}`
  })
}

function legacyFormatCodes(formats: TextFormat[], characterCode: string): string {
  return formats
    .map((format) => {
      switch (format) {
        case 'bold':
          return `${characterCode}l`
        case 'italic':
          return `${characterCode}o`
        case 'underlined':
          return `${characterCode}n`
        case 'strikethrough':
          return `${characterCode}m`
        case 'obfuscated':
          return `${characterCode}k`
      }
    })
    .join('')
}

function formatGradientRuns(
  characters: GradientCharacter[],
  options: GradientOutputOptions,
  formatter: (text: string, colors: string[], formats: TextFormat[]) => string,
): string {
  const output: string[] = []
  let run: GradientCharacter[] = []

  const flush = () => {
    if (!run.length) return
    const text = run.map((character) => character.character).join('')
    const runColors = run
      .map((character) => character.color)
      .filter((color): color is string => color !== null)
    const colors =
      options.simplifyGradients && runColors.length > 1
        ? [runColors[0], runColors[runColors.length - 1]]
        : runColors
    output.push(formatter(text, colors, run[0].formats))
    run = []
  }

  for (const character of characters) {
    if (character.newline) {
      flush()
      output.push('\n')
      continue
    }
    if (
      run.length &&
      (run[0].formats.join(',') !== character.formats.join(',') || character.color === null)
    ) {
      flush()
    }
    run.push(character)
  }
  flush()
  return output.join('')
}

function wrapFormats(text: string, formats: TextFormat[], formatMap: FormatMap): string {
  return formats.reduce((result, format) => {
    const [start, end] = formatMap[format]
    return `${start}${result}${end}`
  }, text)
}

function toTextComponents(characters: GradientCharacter[]) {
  return characters.map((character) => {
    const component: Record<string, string | boolean> = { text: character.character }
    if (character.color) component.color = character.color
    for (const format of character.formats) component[format] = true
    return component
  })
}

function toSnbt(characters: GradientCharacter[]): string {
  return `[${characters
    .map((character) => {
      const entries = [`text:${quoteSnbt(character.character)}`]
      if (character.color) entries.push(`color:${quoteSnbt(character.color)}`)
      for (const format of character.formats) entries.push(`${format}:true`)
      return `{${entries.join(',')}}`
    })
    .join(',')}]`
}

function toHtml(characters: GradientCharacter[]): string {
  const lines: string[] = ['']
  for (const character of characters) {
    if (character.newline) {
      lines.push('')
      continue
    }
    const content = character.character === ' ' ? '&nbsp;' : escapeHtml(character.character)
    const colored = character.color
      ? `<span style="color: ${character.color};">${content}</span>`
      : content
    lines[lines.length - 1] += wrapFormats(colored, character.formats, htmlFormats)
  }
  return lines.map((line) => `<p>${line}</p>`).join('')
}

function toCsv(characters: GradientCharacter[]): string {
  const header = 'color,char,bold,italic,underlined,strikethrough,obfuscated'
  const rows = characters.map((character) =>
    [
      character.color ?? '',
      character.newline ? '\\n' : character.character,
      ...TEXT_FORMATS.map((format) => character.formats.includes(format)),
    ]
      .map(csvCell)
      .join(','),
  )
  return [header, ...rows].join('\n')
}

function csvCell(value: string | boolean): string {
  const text = String(value)
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text
}

function quoteSnbt(value: string): string {
  return `'${value.replaceAll('\\', '\\\\').replaceAll("'", "\\'")}'`
}

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

function hexToColor(hex: string): GradientColor {
  const value = hex.slice(1)
  return {
    red: Number.parseInt(value.slice(0, 2), 16),
    green: Number.parseInt(value.slice(2, 4), 16),
    blue: Number.parseInt(value.slice(4, 6), 16),
  }
}

function colorToHex(color: GradientColor): string {
  return `#${[color.red, color.green, color.blue]
    .map((component) => Math.max(0, Math.min(255, component)).toString(16).padStart(2, '0'))
    .join('')
    .toUpperCase()}`
}
