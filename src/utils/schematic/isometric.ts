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
import { drawTexture, resolveTexture, type BlockAtlas } from './textures'

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
  /** 方块贴图图集；为空或柱体过多时退回纯色 */
  atlas?: BlockAtlas | null
}

/** 超过这个柱体数就不贴图（贴图每面一次 drawImage，纯色快一个量级） */
const MAX_TEXTURED_COLUMNS = 12000

/** 图集内单个贴图的边长（原版方块贴图统一 16×16） */
const tileSrc = 16

/**
 * 画一个侧面：先按平行四边形映射贴图，再叠一层暗色体现光照。
 *
 * 侧面四点满足 P(u, v) = 起点 + u * (dxU, dyU) + v * (0, lift)，
 * 因此变换矩阵取 [dxU, dyU, 0, lift]，源图 16×16 恰好铺满该四边形。
 */
function drawFace(
  ctx: CanvasRenderingContext2D,
  atlas: BlockAtlas | null,
  texture: [number, number, number, number] | null,
  corners: Array<[number, number]>,
  fallbackColor: string,
  darken: number,
  dxU: number,
  dyU: number,
  lift: number,
): void {
  const [start] = corners
  if (atlas && texture && lift > 0) {
    ctx.save()
    ctx.transform(dxU / tileSrc, dyU / tileSrc, 0, lift / tileSrc, start[0], start[1])
    drawTexture(ctx, atlas, texture, 0, 0, tileSrc, tileSrc)
    ctx.restore()
  } else {
    ctx.fillStyle = fallbackColor
    ctx.beginPath()
    ctx.moveTo(corners[0][0], corners[0][1])
    for (let i = 1; i < corners.length; i++) ctx.lineTo(corners[i][0], corners[i][1])
    ctx.closePath()
    ctx.fill()
  }
  ctx.fillStyle = `rgba(15, 23, 42, ${darken})`
  ctx.beginPath()
  ctx.moveTo(corners[0][0], corners[0][1])
  for (let i = 1; i < corners.length; i++) ctx.lineTo(corners[i][0], corners[i][1])
  ctx.closePath()
  ctx.fill()
}

/** 渲染等轴测图到画布；返回实际绘制的柱体数 */
export function drawIsometric(canvas: HTMLCanvasElement, options: IsometricOptions): number {
  const { grid, palette, emptyIndex, worldHeight, canvasWidth, canvasHeight, atlas } = options
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

  let columns = 0
  if (atlas) {
    for (let i = 0; i < grid.topHeight.length; i++) {
      if (grid.topHeight[i] && grid.topIndex[i] !== emptyIndex) columns++
    }
  }
  const atlasReady = atlas && columns > 0 && columns <= MAX_TEXTURED_COLUMNS ? atlas : null
  if (atlasReady) ctx.imageSmoothingEnabled = false

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
      const texture = atlasReady ? resolveTexture(atlasReady, id) : null

      // 左侧面
      drawFace(ctx, atlasReady, texture, [
        [sx - half, sy],
        [sx, sy + quarter],
        [sx, sy + quarter + lift],
        [sx - half, sy + lift],
      ], shade(color, 0.72), 0.26, half, quarter, lift)

      // 右侧面
      drawFace(ctx, atlasReady, texture, [
        [sx, sy + quarter],
        [sx + half, sy],
        [sx + half, sy + lift],
        [sx, sy + quarter + lift],
      ], shade(color, 0.55), 0.42, half, -quarter, lift)

      // 顶面
      if (atlasReady && texture) {
        ctx.save()
        // 单位正方形经该矩阵变换即为 2:1 菱形
        ctx.transform(half / tileSrc, quarter / tileSrc, -half / tileSrc, quarter / tileSrc, sx, sy - quarter)
        drawTexture(ctx, atlasReady, texture, 0, 0, tileSrc, tileSrc)
        ctx.restore()
      } else {
        ctx.fillStyle = color
        ctx.beginPath()
        ctx.moveTo(sx, sy - quarter)
        ctx.lineTo(sx + half, sy)
        ctx.lineTo(sx, sy + quarter)
        ctx.lineTo(sx - half, sy)
        ctx.closePath()
        ctx.fill()
      }

      drawn++
    }
  }
  return drawn
}
