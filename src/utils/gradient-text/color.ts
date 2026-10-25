/**
 * 渐变文字工具 - 颜色运算
 *
 * 十六进制解析与归一化、色标线性插值、Minecraft 阴影色（原版规则：阴影取
 * 同色相同饱和度、亮度为 RGB 最大值四分之一）。
 */
import type { GradientColor } from './types'

const HEX_PATTERN = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i
const HEX_SCAN = /#(?:[0-9a-f]{3}|[0-9a-f]{6})\b/gi
const RGB_SCAN = /rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})/gi

/** 归一化为 `#RRGGBB`（大写），非法输入返回 null */
export function normalizeHexColor(value: string): string | null {
  const matched = value.trim().match(HEX_PATTERN)
  if (!matched) return null
  const digits = matched[1]
  const expanded =
    digits.length === 3
      ? digits.replace(/./g, (digit) => digit + digit)
      : digits
  return `#${expanded.toUpperCase()}`
}

function channelsOf(hex: string): GradientColor {
  const digits = hex.slice(1)
  return {
    red: Number.parseInt(digits.slice(0, 2), 16),
    green: Number.parseInt(digits.slice(2, 4), 16),
    blue: Number.parseInt(digits.slice(4, 6), 16),
  }
}

function hexOf(color: GradientColor): string {
  const channel = (value: number) =>
    Math.max(0, Math.min(255, Math.round(value)))
      .toString(16)
      .padStart(2, '0')
      .toUpperCase()
  return `#${channel(color.red)}${channel(color.green)}${channel(color.blue)}`
}

/** 从任意文本里收集色标：先十六进制，后 rgb()/rgba()，去重保序 */
export function parseGradientColors(value: string): string[] {
  const collected: string[] = []
  for (const matched of value.matchAll(HEX_SCAN)) {
    const color = normalizeHexColor(matched[0])
    if (color) collected.push(color)
  }
  for (const matched of value.matchAll(RGB_SCAN)) {
    const [red, green, blue] = matched.slice(1, 4).map(Number)
    if ([red, green, blue].every((channel) => channel >= 0 && channel <= 255)) {
      collected.push(hexOf({ red, green, blue }))
    }
  }
  return [...new Set(collected)]
}

/** 把色标线性插值成 count 个颜色 */
export function interpolateGradient(colors: string[], count: number): string[] {
  const stops = colors
    .map((color) => normalizeHexColor(color))
    .filter((color): color is string => color !== null)
  if (count <= 0 || stops.length === 0) return []
  if (stops.length === 1 || count === 1) {
    return Array.from({ length: count }, () => stops[0])
  }

  const rgbStops = stops.map(channelsOf)
  return Array.from({ length: count }, (_, index) => {
    const scaled = (index / (count - 1)) * (rgbStops.length - 1)
    const left = Math.floor(scaled)
    const right = Math.min(rgbStops.length - 1, left + 1)
    const ratio = scaled - left
    const from = rgbStops[left]
    const to = rgbStops[right]
    return hexOf({
      red: from.red + (to.red - from.red) * ratio,
      green: from.green + (to.green - from.green) * ratio,
      blue: from.blue + (to.blue - from.blue) * ratio,
    })
  })
}

/** 标准 HSL → RGB */
function hslToRgb(hue: number, saturation: number, lightness: number): GradientColor {
  const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation
  const section = hue / 60
  const second = chroma * (1 - Math.abs((section % 2) - 1))
  const offset = lightness - chroma / 2
  const [red, green, blue] =
    section < 1
      ? [chroma, second, 0]
      : section < 2
        ? [second, chroma, 0]
        : section < 3
          ? [0, chroma, second]
          : section < 4
            ? [0, second, chroma]
            : section < 5
              ? [second, 0, chroma]
              : [chroma, 0, second]
  return {
    red: (red + offset) * 255,
    green: (green + offset) * 255,
    blue: (blue + offset) * 255,
  }
}

/** Minecraft 阴影色；无色或非法颜色返回纯黑 */
export function getMinecraftTextShadow(color: string | null): string {
  const normalized = color ? normalizeHexColor(color) : null
  if (!normalized) return '#000000'

  const { red, green, blue } = channelsOf(normalized)
  const maximum = Math.max(red, green, blue) / 255
  const minimum = Math.min(red, green, blue) / 255
  const delta = maximum - minimum
  const saturation = maximum === 0 ? 0 : delta / maximum

  let hue = 0
  if (delta !== 0) {
    const r = red / 255
    const g = green / 255
    const b = blue / 255
    if (maximum === r) {
      hue = ((g - b) / delta) % 6
    } else if (maximum === g) {
      hue = (b - r) / delta + 2
    } else {
      hue = (r - g) / delta + 4
    }
    hue = (hue * 60 + 360) % 360
  }

  return hexOf(hslToRgb(hue, saturation, maximum / 4))
}
