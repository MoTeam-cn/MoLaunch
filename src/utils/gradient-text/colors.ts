/**
 * 渐变文字工具 - 颜色（对应 specs/cleanroom/gradient-text.md §2.3 §3 §4）
 */
import type { GradientCell } from './types'

const HEX_INPUT = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i
const HEX_SCAN = /#(?:[0-9a-f]{3}|[0-9a-f]{6})\b/gi
const RGB_SCAN = /rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})/gi

type Rgb = { r: number; g: number; b: number }

/** §3 归一化：`#RGB`/`#RRGGBB` → `#RRGGBB`（大写），非法返回 null */
export function normalizeHexColor(value: string): string | null {
  const matched = value.trim().match(HEX_INPUT)
  if (!matched) return null
  const digits = matched[1]
  const full = digits.length === 3 ? digits.replace(/./g, (d) => d + d) : digits
  return `#${full.toUpperCase()}`
}

function toRgb(hex: string): Rgb {
  const digits = hex.slice(1)
  return {
    r: Number.parseInt(digits.slice(0, 2), 16),
    g: Number.parseInt(digits.slice(2, 4), 16),
    b: Number.parseInt(digits.slice(4, 6), 16),
  }
}

function fromRgb(rgb: Rgb): string {
  const byte = (value: number) =>
    Math.max(0, Math.min(255, Math.round(value))).toString(16).padStart(2, '0').toUpperCase()
  return `#${byte(rgb.r)}${byte(rgb.g)}${byte(rgb.b)}`
}

/** §3 从自由文本提取色标：先十六进制、后 rgb()/rgba()，去重保序 */
export function parseGradientColors(value: string): string[] {
  const found: string[] = []
  for (const hit of value.matchAll(HEX_SCAN)) {
    const hex = normalizeHexColor(hit[0])
    if (hex) found.push(hex)
  }
  for (const hit of value.matchAll(RGB_SCAN)) {
    const [r, g, b] = hit.slice(1, 4).map(Number)
    if ([r, g, b].every((channel) => channel >= 0 && channel <= 255)) {
      found.push(fromRgb({ r, g, b }))
    }
  }
  return [...new Set(found)]
}

/** §2.3 等距线性插值：N 个颜色覆盖色标序列，端点必取 */
export function interpolateGradient(colors: string[], count: number): string[] {
  const stops = colors
    .map((color) => normalizeHexColor(color))
    .filter((color): color is string => color !== null)
  if (count <= 0 || stops.length === 0) return []
  if (stops.length === 1 || count === 1) return Array.from({ length: count }, () => stops[0])

  const rgbStops = stops.map(toRgb)
  return Array.from({ length: count }, (_, index) => {
    const position = (index / (count - 1)) * (rgbStops.length - 1)
    const left = Math.floor(position)
    const right = Math.min(rgbStops.length - 1, left + 1)
    const ratio = position - left
    const a = rgbStops[left]
    const b = rgbStops[right]
    return fromRgb({
      r: a.r + (b.r - a.r) * ratio,
      g: a.g + (b.g - a.g) * ratio,
      b: a.b + (b.b - a.b) * ratio,
    })
  })
}

/** 标准 HSL → RGB（hue 单位：度） */
function hslToRgb(hue: number, saturation: number, lightness: number): Rgb {
  const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation
  const sector = hue / 60
  const second = chroma * (1 - Math.abs((sector % 2) - 1))
  const base = lightness - chroma / 2
  const [r, g, b] =
    sector < 1
      ? [chroma, second, 0]
      : sector < 2
        ? [second, chroma, 0]
        : sector < 3
          ? [0, chroma, second]
          : sector < 4
            ? [0, second, chroma]
            : sector < 5
              ? [second, 0, chroma]
              : [chroma, 0, second]
  return { r: (r + base) * 255, g: (g + base) * 255, b: (b + base) * 255 }
}

/** §4 原版阴影色：同色相同饱和度，亮度 = 通道最大值 / 4 */
export function getMinecraftTextShadow(color: string | null): string {
  const normalized = color ? normalizeHexColor(color) : null
  if (!normalized) return '#000000'

  const rgb = toRgb(normalized)
  const maximum = Math.max(rgb.r, rgb.g, rgb.b) / 255
  const minimum = Math.min(rgb.r, rgb.g, rgb.b) / 255
  const delta = maximum - minimum
  const saturation = maximum === 0 ? 0 : delta / maximum

  let hue = 0
  if (delta !== 0) {
    const r = rgb.r / 255
    const g = rgb.g / 255
    const b = rgb.b / 255
    if (maximum === r) hue = ((g - b) / delta) % 6
    else if (maximum === g) hue = (b - r) / delta + 2
    else hue = (r - g) / delta + 4
    hue = (hue * 60 + 360) % 360
  }

  return fromRgb(hslToRgb(hue, saturation, maximum / 4))
}

/** §2.4 可见字符判定：空白不消耗色标 */
export function isVisibleGlyph(cell: GradientCell): boolean {
  return !cell.newline && !/\s/u.test(cell.character)
}
