/**
 * 投影等轴测立体预览
 *
 * 把俯视高度图（每列最高方块 + 高度）画成 2:1 等轴测柱体阵列，
 * 让用户一眼看出整体轮廓与高度分布，而不是逐层翻页。
 *
 * 约定：格子 (col, row) 对应世界 (x = col * step, z = row * step)，
 * topHeight 为相对方块数（0 表示该列全空）。
 */

import { blockColor, shade, type SchematicOverview } from './decode'

/** 单张图最多绘制的柱体数（超出按整数倍抽稀） */
const MAX_CELLS = 20000

/** 抽稀后的概览（相邻格子合并，取最高者） */
export interface OverviewGrid {
  width: number
  height: number
  topIndex: Uint16Array
  topHeight: Uint16Array
}

/** 按上限抽稀俯视高度图 */
export function compressOverview(overview: SchematicOverview): OverviewGrid {
  const { width, height, topIndex, topHeight } = overview
  const cells = width * height
  if (cells <= MAX_CELLS) {
    return { width, height, topIndex, topHeight }
  }
  const factor = Math.ceil(Math.sqrt(cells / MAX_CELLS))
  const outWidth = Math.ceil(width / factor)
  const outHeight = Math.ceil(height / factor)
  const outIndex = new Uint16Array(outWidth * outHeight)
  const outHeightArr = new Uint16Array(outWidth * outHeight)

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const source = row * width + col
      const h = topHeight[source]
      if (!h) continue
      const target = Math.floor(row / factor) * outWidth + Math.floor(col / factor)
      if (h >= outHeightArr[target]) {
        outHeightArr[target] = h
        outIndex[target] = topIndex[source]
      }
    }
  }
  return { width: outWidth, height: outHeight, topIndex: outIndex, topHeight: outHeightArr }
}

export interface IsometricOptions {
  grid: OverviewGrid
  palette: readonly string[]
  /** 表示"该列全空"的索引 */
  emptyIndex: number
  /** 世界总高度，用于把高度归一化 */
  worldHeight: number
  canvasWidth: number
  canvasHeight: number
}

/** 渲染等轴测图到画布；返回实际绘制的柱体数 */
export function drawIsometric(canvas: HTMLCanvasElement, options: IsometricOptions): number {
  const { grid, palette, emptyIndex, worldHeight, canvasWidth, canvasHeight } = options
  const ctx = canvas.getContext('2d')
  if (!ctx) return 0

  canvas.width = Math.max(Math.floor(canvasWidth), 1)
  canvas.height = Math.max(Math.floor(canvasHeight), 1)
  ctx.clearRect(0, 0, canvas.width, canvas.height)

  const pad = 12
  const span = grid.width + grid.height
  if (span <= 0) return 0

  // 2:1 等轴测：横向占满可用宽度
  const tileW = Math.max(((canvasWidth - pad * 2) * 2) / span, 2)
  const tileH = tileW / 2
  const maxLift = Math.max(canvasHeight - pad * 2 - ((span - 2) * tileH) / 2 - tileH, tileH)

  const centerX = canvas.width / 2
  const baseY = pad + maxLift

  let drawn = 0
  // 画家算法：按 (col + row) 递增，保证近处覆盖远处
  const diagonalMax = grid.width - 1 + (grid.height - 1)
  for (let diagonal = 0; diagonal <= diagonalMax; diagonal++) {
    const colStart = Math.max(0, diagonal - (grid.height - 1))
    const colEnd = Math.min(diagonal, grid.width - 1)
    for (let col = colStart; col <= colEnd; col++) {
      const row = diagonal - col
      const cell = row * grid.width + col
      const heightValue = grid.topHeight[cell]
      if (!heightValue) continue
      const index = grid.topIndex[cell]
      if (index === emptyIndex) continue
      const id = palette[index]
      if (!id || id === 'minecraft:air') continue

      const ratio = worldHeight > 0 ? Math.min(heightValue / worldHeight, 1) : 0
      const lift = tileH * 0.6 + ratio * (maxLift - tileH * 0.6)
      const sx = centerX + ((col - row) * tileW) / 2
      const sy = baseY + ((col + row) * tileH) / 2 - lift

      const color = blockColor(id)
      const half = tileW / 2
      const quarter = tileH / 2

      // 左侧面
      ctx.fillStyle = shade(color, 0.72)
      ctx.beginPath()
      ctx.moveTo(sx - half, sy)
      ctx.lineTo(sx, sy + quarter)
      ctx.lineTo(sx, sy + quarter + lift)
      ctx.lineTo(sx - half, sy + lift)
      ctx.closePath()
      ctx.fill()

      // 右侧面
      ctx.fillStyle = shade(color, 0.55)
      ctx.beginPath()
      ctx.moveTo(sx, sy + quarter)
      ctx.lineTo(sx + half, sy)
      ctx.lineTo(sx + half, sy + lift)
      ctx.lineTo(sx, sy + quarter + lift)
      ctx.closePath()
      ctx.fill()

      // 顶面
      ctx.fillStyle = color
      ctx.beginPath()
      ctx.moveTo(sx, sy - quarter)
      ctx.lineTo(sx + half, sy)
      ctx.lineTo(sx, sy + quarter)
      ctx.lineTo(sx - half, sy)
      ctx.closePath()
      ctx.fill()

      drawn++
    }
  }
  return drawn
}
