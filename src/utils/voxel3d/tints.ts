/**
 * 生物群系着色（tint）
 *
 * 草方块顶面、树叶、水这些贴图本身是灰度的，颜色靠渲染时乘上一个 tint。
 * 这里用平原群系的默认取值，足以让颜色正确；不追求按群系逐个还原。
 */
export type Rgb = [number, number, number]

/** 草地：平原 */
const GRASS: Rgb = [0x91 / 255, 0xbd / 255, 0x59 / 255]
/** 树叶：多数树种共用 */
const FOLIAGE: Rgb = [0x59 / 255, 0xae / 255, 0x30 / 255]
/** 水 */
const WATER: Rgb = [0x3f / 255, 0x76 / 255, 0xe4 / 255]

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
  return [1, 1, 1]
}
