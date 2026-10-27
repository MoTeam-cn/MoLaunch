/**
 * 生物群系着色（tint）
 *
 * 草方块顶面、树叶、水这些贴图本身是灰度的，颜色靠渲染时乘上一个 tint。
 * 这里用平原群系的默认取值，足以让颜色正确；不追求按群系逐个还原。
 *
 * 顶点色在 three.js 里按线性空间使用，不做 sRGB 转换，所以色号在这里就要转成线性，
 * 否则草地上屏会偏亮偏艳。
 */
export type Rgb = [number, number, number]

/** sRGB 分量转线性 */
function linear(srgb: number): number {
  return srgb <= 0.04045 ? srgb / 12.92 : ((srgb + 0.055) / 1.055) ** 2.4
}

function fromHex(hex: number): Rgb {
  return [
    linear(((hex >> 16) & 0xff) / 255),
    linear(((hex >> 8) & 0xff) / 255),
    linear((hex & 0xff) / 255),
  ]
}

/** 草地：平原 */
const GRASS = fromHex(0x91bd59)
/** 树叶：多数树种共用 */
const FOLIAGE = fromHex(0x59ae30)
/** 水 */
const WATER = fromHex(0x3f76e4)
/** 有 tintindex 但拿不准色号时保持原色 */
const NEUTRAL: Rgb = [1, 1, 1]

const GRASS_BLOCKS = new Set([
  'grass_block',
  'grass',
  'short_grass',
  'tall_grass',
  'fern',
  'large_fern',
  'vine',
  'sugar_cane',
  'potted_fern',
])

/**
 * 给标了 tintindex 的面挑颜色。
 *
 * 返回 null 表示不改色；返回白色表示「有 tintindex 但不知道具体色号」，保持原样。
 */
export function resolveTint(blockName: string, tintindex: number | undefined): Rgb | null {
  if (tintindex === undefined || tintindex < 0) return null
  if (GRASS_BLOCKS.has(blockName)) return GRASS
  if (blockName.endsWith('_leaves')) return FOLIAGE
  if (blockName === 'water' || blockName === 'bubble_column') return WATER
  return NEUTRAL
}
