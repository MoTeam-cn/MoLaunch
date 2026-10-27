/**
 * 投影数据解码与切片
 *
 * 后端返回 base64 的 u16 小端方块索引数组，索引顺序为 x 最快、其次 z、最后 y。
 * 本模块负责解码、按轴切片与方块配色（已知方块查表，未知方块按 id 哈希取色）。
 */

export type SliceAxis = 'y' | 'x' | 'z'

export interface SchematicSlice {
  width: number
  height: number
  /** 行优先的方块索引（0 表示调色板第 0 项） */
  cells: Uint16Array
}

/** base64 → u16 数组 */
export function decodeBlocks(base64: string): Uint16Array {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  const count = Math.floor(bytes.length / 2)
  const out = new Uint16Array(count)
  for (let i = 0; i < count; i++) {
    out[i] = bytes[i * 2] | (bytes[i * 2 + 1] << 8)
  }
  return out
}

/** 指定轴的层数与切片宽高 */
export function sliceBounds(
  size: readonly [number, number, number],
  axis: SliceAxis,
): { layers: number; width: number; height: number } {
  const [sx, sy, sz] = size
  if (axis === 'x') return { layers: sx, width: sz, height: sy }
  if (axis === 'z') return { layers: sz, width: sx, height: sy }
  return { layers: sy, width: sx, height: sz }
}

/** 三维索引（x 最快、其次 z、最后 y） */
export function blockIndex(
  size: readonly [number, number, number],
  x: number,
  y: number,
  z: number,
): number {
  const [sx, , sz] = size
  return x + z * sx + y * sx * sz
}

/** 抽取某一层，返回行优先二维网格 */
export function extractSlice(
  blocks: Uint16Array,
  size: readonly [number, number, number],
  axis: SliceAxis,
  layer: number,
): SchematicSlice {
  const { width, height } = sliceBounds(size, axis)
  const cells = new Uint16Array(width * height)
  const [sx, sy, sz] = size
  const clamped = Math.min(Math.max(layer, 0), Math.max((axis === 'x' ? sx : axis === 'z' ? sz : sy) - 1, 0))

  for (let v = 0; v < height; v++) {
    for (let u = 0; u < width; u++) {
      let index = 0
      if (axis === 'y') {
        index = blockIndex(size, u, clamped, v)
      } else if (axis === 'x') {
        index = blockIndex(size, clamped, v, u)
      } else {
        index = blockIndex(size, u, v, clamped)
      }
      cells[v * width + u] = blocks[index] ?? 0
    }
  }
  return { width, height, cells }
}

/** 已知方块配色表（常用方块，未命中走哈希取色） */
const BLOCK_COLORS: Record<string, string> = {
  'minecraft:air': 'transparent',
  'minecraft:stone': '#7d7d7d',
  'minecraft:cobblestone': '#6e6e6e',
  'minecraft:dirt': '#8a5a3b',
  'minecraft:grass_block': '#5b9c3a',
  'minecraft:sand': '#dbd3a0',
  'minecraft:gravel': '#8b8682',
  'minecraft:oak_planks': '#b08a52',
  'minecraft:spruce_planks': '#7a5b35',
  'minecraft:birch_planks': '#d7c185',
  'minecraft:oak_log': '#6b5334',
  'minecraft:oak_leaves': '#4a7a2c',
  'minecraft:water': '#3f76e4',
  'minecraft:lava': '#ea6a12',
  'minecraft:glass': '#c8e6f0',
  'minecraft:sandstone': '#d8cb92',
  'minecraft:bricks': '#9a5b48',
  'minecraft:stone_bricks': '#7a7a7a',
  'minecraft:deepslate': '#4a4a4f',
  'minecraft:bedrock': '#2b2b2b',
  'minecraft:snow_block': '#f3f7f7',
  'minecraft:ice': '#a5c8f0',
  'minecraft:netherrack': '#6b2f2f',
  'minecraft:soul_sand': '#54402f',
  'minecraft:obsidian': '#1a1226',
  'minecraft:torch': '#f0c040',
  'minecraft:chest': '#9a6f3b',
  'minecraft:crafting_table': '#9c7040',
  'minecraft:furnace': '#6b6b6b',
  'minecraft:white_wool': '#e9ecec',
  'minecraft:red_wool': '#a02722',
  'minecraft:blue_wool': '#35399d',
  'minecraft:green_wool': '#546d1b',
  'minecraft:black_wool': '#1d1d21',
}

const colorCache = new Map<string, string>()

/** 去掉 blockstate 属性，只留方块名 */
export function blockBaseName(id: string): string {
  const open = id.indexOf('[')
  return open < 0 ? id : id.slice(0, open)
}

/** 方块 id → 展示色（未知方块按 id 哈希生成稳定颜色） */
export function blockColor(id: string): string {
  const base = blockBaseName(id)
  const cached = colorCache.get(base)
  if (cached) return cached
  const known = BLOCK_COLORS[base]
  if (known) {
    colorCache.set(base, known)
    return known
  }
  let hash = 0
  for (let i = 0; i < base.length; i++) {
    hash = (hash * 31 + base.charCodeAt(i)) % 360
  }
  const color = `hsl(${hash}, 42%, 58%)`
  colorCache.set(base, color)
  return color
}

/** 该切片中的非空气方块数 */
export function countSliceBlocks(
  cells: Uint16Array,
  palette: readonly string[],
  airIndex: number,
): number {
  let count = 0
  for (let i = 0; i < cells.length; i++) {
    if (cells[i] !== airIndex && palette[cells[i]] !== undefined) count++
  }
  return count
}

/** 调色板中空气对应的索引（找不到返回 -1） */
export function findAirIndex(palette: readonly string[]): number {
  return palette.findIndex((id) => blockBaseName(id) === 'minecraft:air')
}

/** 调整颜色明暗：factor < 1 变暗，> 1 变亮（支持 #rrggbb 与 hsl()） */
export function shade(color: string, factor: number): string {
  if (color.startsWith('#')) {
    const hex = color.slice(1)
    const full = hex.length === 3 ? hex.split('').map((c) => c + c).join('') : hex
    const value = Number.parseInt(full, 16)
    if (Number.isNaN(value)) return color
    const channels = [(value >> 16) & 0xff, (value >> 8) & 0xff, value & 0xff]
    const scaled = channels.map((c) => Math.max(0, Math.min(255, Math.round(c * factor))))
    return `#${scaled.map((c) => c.toString(16).padStart(2, '0')).join('')}`
  }
  const match = color.match(/hsl\(([\d.]+),\s*([\d.]+)%,\s*([\d.]+)%\)/)
  if (!match) return color
  const lightness = Math.max(0, Math.min(100, Number(match[3]) * factor))
  return `hsl(${match[1]}, ${match[2]}%, ${lightness.toFixed(1)}%)`
}

/** 俯视概览数据 */
export interface SchematicOverview {
  width: number
  height: number
  step: number
  /** 每列最高方块的调色板索引 */
  topIndex: Uint16Array
  /** 每列最高方块的高度（0 表示全空） */
  topHeight: Uint16Array
  /** 每层非空方块数 */
  layerCounts: number[]
  /** 表示"该列全空"的方块索引 */
  emptyIndex: number
}

/** 解压 zlib 流并解码为 u16 小端数组（3D 体素数据用） */
export async function decodeZlibU16(base64: string): Promise<Uint16Array> {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  const stream = new DecompressionStream('deflate')
  const writer = stream.writable.getWriter()
  void writer.write(bytes)
  void writer.close()
  const buffer = await new Response(stream.readable).arrayBuffer()
  return new Uint16Array(buffer)
}

/** 解码概览里的 u16 小端数组 */
export function decodeU16(base64: string): Uint16Array {
  const binary = atob(base64)
  const out = new Uint16Array(binary.length >> 1)
  for (let i = 0; i < out.length; i++) {
    out[i] = binary.charCodeAt(i * 2) | (binary.charCodeAt(i * 2 + 1) << 8)
  }
  return out
}

/** 最密集的非空层（找不到返回 -1） */
export function densestLayer(layerCounts: readonly number[]): number {
  let best = -1
  let bestCount = 0
  layerCounts.forEach((count, index) => {
    if (count > bestCount) {
      bestCount = count
      best = index
    }
  })
  return best
}

/**
 * 打开投影时该定位到的层。
 *
 * 带地形的投影（地表建筑）最密集的层在地底——整层石头，直接跳过去只能看到一片岩层。
 * 这里取「密度不低于峰值一半的最上层」，对地形投影正好落在最上面那层完整地面，
 * 对悬空建筑则落在实体最多的那一层。
 */
export function surfaceLayer(layerCounts: readonly number[]): number {
  let peak = 0
  for (const count of layerCounts) {
    if (count > peak) peak = count
  }
  if (peak === 0) return -1
  const threshold = peak * 0.5
  for (let index = layerCounts.length - 1; index >= 0; index--) {
    if (layerCounts[index] >= threshold) return index
  }
  return -1
}

/** 距离指定层最近的非空层（找不到返回 -1） */
export function nearestOccupiedLayer(layerCounts: readonly number[], from: number): number {
  if (layerCounts[from]) return from
  for (let offset = 1; offset < layerCounts.length; offset++) {
    const up = from + offset
    if (up < layerCounts.length && layerCounts[up]) return up
    const down = from - offset
    if (down >= 0 && layerCounts[down]) return down
  }
  return -1
}
