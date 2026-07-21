/**
 * 颜色工具：HEX ↔ RGB ↔ HSL 转换 + 主色阶生成
 *
 * 用户仅选择一个主色（500），由本工具生成 50~950 共 11 档色阶，
 * 写入 CSS 变量供 Tailwind `primary-*` 与 `main.css` 中所有 `var(--color-primary-*)` 消费。
 *
 * 算法参考 Arco Design 色板生成思路：基于 HSL 调整 L（亮度）生成档位，
 * 50/100 极亮（接近白），900/950 极暗（接近黑），中间档按曲线分布。
 */

/** HEX → RGB（返回 [r, g, b]，0-255） */
export function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace('#', '').trim()
  // 支持 3 位缩写：#abc → #aabbcc
  const full = clean.length === 3
    ? clean.split('').map((c) => c + c).join('')
    : clean
  const num = parseInt(full, 16)
  if (Number.isNaN(num) || full.length !== 6) {
    return [22, 93, 255] // 兜底：Arco 蓝 #165dff
  }
  return [(num >> 16) & 0xff, (num >> 8) & 0xff, num & 0xff]
}

/** RGB → HEX（返回 #rrggbb） */
export function rgbToHex(r: number, g: number, b: number): string {
  const toHex = (n: number) => {
    const clamped = Math.max(0, Math.min(255, Math.round(n)))
    return clamped.toString(16).padStart(2, '0')
  }
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`
}

/** RGB → HSL（h/s/l 均为 0-1） */
export function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  const r1 = r / 255
  const g1 = g / 255
  const b1 = b / 255
  const max = Math.max(r1, g1, b1)
  const min = Math.min(r1, g1, b1)
  let h = 0
  const l = (max + min) / 2
  const d = max - min
  const s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1))
  if (d !== 0) {
    switch (max) {
      case r1: h = ((g1 - b1) / d + (g1 < b1 ? 6 : 0)) / 6; break
      case g1: h = ((b1 - r1) / d + 2) / 6; break
      case b1: h = ((r1 - g1) / d + 4) / 6; break
    }
  }
  return [h, s, l]
}

/** HSL → RGB（返回 [r, g, b]，0-255） */
export function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const hueToRgb = (p: number, q: number, t: number) => {
    let t1 = t
    if (t1 < 0) t1 += 1
    if (t1 > 1) t1 -= 1
    if (t1 < 1 / 6) return p + (q - p) * 6 * t1
    if (t1 < 1 / 2) return q
    if (t1 < 2 / 3) return p + (q - p) * (2 / 3 - t1) * 6
    return p
  }
  if (s === 0) {
    const v = Math.round(l * 255)
    return [v, v, v]
  }
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s
  const p = 2 * l - q
  return [
    Math.round(hueToRgb(p, q, h + 1 / 3) * 255),
    Math.round(hueToRgb(p, q, h) * 255),
    Math.round(hueToRgb(p, q, h - 1 / 3) * 255),
  ]
}

/** HSL → HEX */
export function hslToHex(h: number, s: number, l: number): string {
  const [r, g, b] = hslToRgb(h, s, l)
  return rgbToHex(r, g, b)
}

/**
 * 由主色生成 11 档色阶（50~950）
 *
 * 算法：
 * - 解析主色 HEX → HSL
 * - 保持 H、S 不变（仅极暗/极亮档轻微调 S 防止发灰），按预设 L 表生成各档
 * - L 表参考 Arco Design / Tailwind 标准蓝色色阶的亮度分布
 *
 * @param baseHex 主色 HEX（如 "#165dff"）
 * @returns Record<档位, HEX>，键为 "50"~"950"
 */
export function generateColorScale(baseHex: string): Record<string, string> {
  const [r, g, b] = hexToRgb(baseHex)
  const [h, s] = rgbToHsl(r, g, b)

  // L 表：参考主流色板亮度分布
  // 主色 L 通常在 0.45~0.55 之间，500 档直接用主色
  const lTable: Record<string, number> = {
    '50': 0.97,
    '100': 0.93,
    '200': 0.86,
    '300': 0.76,
    '400': 0.66,
    '500': -1, // 标记：直接用主色
    '600': 0.54,
    '700': 0.44,
    '800': 0.34,
    '900': 0.25,
    '950': 0.18,
  }

  const scale: Record<string, string> = {}
  for (const key of Object.keys(lTable)) {
    if (key === '500') {
      scale[key] = baseHex.toLowerCase()
      continue
    }
    const l = lTable[key]
    // 极亮档（50/100）降低饱和度避免过艳；极暗档（900/950）适度提升饱和度避免发灰
    let adjS = s
    if (l >= 0.9) adjS = Math.max(0.3, s * 0.85)
    else if (l <= 0.25) adjS = Math.min(1, s * 1.05)
    scale[key] = hslToHex(h, adjS, l)
  }
  return scale
}

/**
 * 将色阶写入 document.documentElement 的 CSS 变量
 *
 * 写入：
 * - `--color-primary-50` ~ `--color-primary-950`：HEX 值（如 `#165dff`）
 * - `--color-primary-rgb-50` ~ `--color-primary-rgb-950`：空格分隔的 RGB（如 `22 93 255`），
 *   供 Tailwind `rgb(var(--color-primary-500) / <alpha-value>)` 使用
 * - `--color-primary`：兼容旧定义，值 = 500 档 RGB（空格分隔）
 *
 * @param baseHex 主色 HEX
 */
export function applyPrimaryColor(baseHex: string): void {
  const scale = generateColorScale(baseHex)
  const root = document.documentElement
  for (const [key, hex] of Object.entries(scale)) {
    const [r, g, b] = hexToRgb(hex)
    root.style.setProperty(`--color-primary-${key}`, hex)
    root.style.setProperty(`--color-primary-rgb-${key}`, `${r} ${g} ${b}`)
  }
  // 兼容旧变量（main.css 顶部 :root 中的 --color-primary）
  const [r, g, b] = hexToRgb(scale['500'])
  root.style.setProperty('--color-primary', `${r} ${g} ${b}`)
}

/** 预设主题色（参考 Arco Design 颜色选择器默认色板） */
export const PRESET_COLORS: string[] = [
  '#165dff', // 极客蓝（默认）
  '#00b42a', // 绿
  '#ff7d00', // 橙
  '#f53f3f', // 红
  '#722ed1', // 紫
  '#0fc6c2', // 青
  '#eb2f96', // 洋红
  '#7c3aed', // 深紫
  '#f77234', // 黄
  '#276dd4', // 钴蓝
  '#376b49', // 墨绿
  '#5b5b5b', // 灰
]
