/**
 * 地形阴影渲染（hillshade + terrace + contour）
 *
 * 参考 prompt-frontend.md 的 applyTerrainShading 实现，适配项目坐标系。
 *
 * 光源方向：左上 (azimuth=315°, altitude=30°)
 * - LX=-0.8660254, LY=-0.5, LZ=0.5
 *
 * 渲染效果：
 * 1. Hillshade：根据高度差计算坡度，模拟光照（intensity = 0.55 + 0.45 * max(0, N·L)）
 * 2. Terrace 台阶线：
 *    - isEngraved 模式（scale=1 && pxPerBlock>=2）：方块边界高度突变处画强暗线
 *    - Graded 模式（其他情况）：柔和边界，高度差>2 时按梯度加深
 * 3. Contour：等高线（可选，每 10 格一条，高度差 <0.3 时提亮 40）
 *
 * 性能：纯 JS 双层循环，64×64 tile 约 4096 像素，单 tile <1ms。
 * 在 Worker 内调用，不阻塞主线程。
 *
 * @param rgba         RGBA 图像数据（imgW × imgH × 4），原地修改
 * @param imgW         图像宽度（像素）
 * @param imgH         图像高度（像素）
 * @param heights      高度数组（hw × hh，float，方块单位，来自 cubiomes mapApproxHeight）
 * @param hsx          height 数组宽度（= hw）
 * @param hsz          height 数组高度（= hh）
 * @param heightCellPx 每个 height cell 的像素尺寸（= imgW / hsx）
 * @param options      渲染选项
 */
export interface TerrainShadingOptions {
  /** cubiomes scale（1=精细, 4=粗采样, 16=区块级） */
  scale: number
  /** 每个 biome cell 的像素数（= imgW / sx） */
  pixelsPerCell: number
  /** 是否绘制等高线（默认 false） */
  doContour?: boolean
  /** 最大渲染高度（超过此高度+8 的区域变暗，默认 Infinity=不限） */
  ymax?: number
}

export function applyTerrainShading(
  rgba: Uint8ClampedArray,
  imgW: number,
  imgH: number,
  heights: Float32Array,
  hsx: number,
  hsz: number,
  heightCellPx: number,
  options: TerrainShadingOptions,
): void {
  const { scale, pixelsPerCell, doContour = false, ymax = Infinity } = options

  // 光源方向：左上 (azimuth=315°, altitude=30°)
  const LX = -0.8660254
  const LY = -0.5
  const LZ = 0.5
  const contourInterval = 10

  // isEngraved 模式：scale=1 且每个方块≥2像素时，画强台阶线（方块边界分明）
  // Graded 模式：其他情况，画柔和边界（高度差>2 时按梯度加深）
  const pxPerBlock = pixelsPerCell / (scale === 1 ? 1 : scale)
  const isEngraved = scale === 1 && pxPerBlock >= 2

  for (let py = 0; py < imgH; py++) {
    for (let px = 0; px < imgW; px++) {
      const idx = (py * imgW + px) * 4

      // 像素对应的 height cell 索引
      const bx = Math.floor(px / heightCellPx)
      const bz = Math.floor(py / heightCellPx)

      // 块内偏移（用于 terrace 边界检测，0~1）
      const ox = (px % heightCellPx) / heightCellPx
      const oy = (py % heightCellPx) / heightCellPx

      // 采样高度（clamp 到有效范围，超出用 64 = 海平面）
      // 防御性边界：hsx/hsz 较小时 hsx-2 可能为负，用 Math.max(0, hsx-2) 确保下界不为负
      const cx = Math.min(Math.max(bx, 0), Math.max(0, hsx - 2))
      const cz = Math.min(Math.max(bz, 0), Math.max(0, hsz - 2))
      const h00 = heights[cz * hsx + cx] ?? 64
      const h10 = heights[cz * hsx + Math.min(cx + 1, hsx - 1)] ?? h00
      const h01 = heights[Math.min(cz + 1, hsz - 1) * hsx + cx] ?? h00

      // 坡度（有限差分）
      const dx = h10 - h00
      const dz = h01 - h00

      // hillshade：法向量 N = (-dx, 1, -dz) / len，光向量 L = (LX, LZ, LY)
      const len = Math.sqrt(dx * dx + dz * dz + 1)
      const nx = -dx / len
      const nz = -dz / len
      const ny = 1 / len
      let intensity = nx * LX + ny * LZ + nz * LY
      intensity = 0.55 + 0.45 * Math.max(0, Math.min(1, intensity))

      // ymax 限制：超过 ymax+8 的区域变暗（模拟雪线/高处阴影）
      if (ymax < Infinity && h00 > ymax + 8) intensity *= 0.3

      // 应用亮度
      rgba[idx] = Math.round(rgba[idx] * intensity)
      rgba[idx + 1] = Math.round(rgba[idx + 1] * intensity)
      rgba[idx + 2] = Math.round(rgba[idx + 2] * intensity)

      // Terrace 台阶线
      if (isEngraved) {
        // 强台阶线：方块边界高度突变 >0.01 处加深 35%
        if (cx > 0) {
          const hL = heights[cz * hsx + (cx - 1)] ?? h00
          if (Math.abs(h00 - hL) > 0.01 && (ox < 0.08 || ox > 0.92)) {
            rgba[idx] = Math.round(rgba[idx] * 0.65)
            rgba[idx + 1] = Math.round(rgba[idx + 1] * 0.65)
            rgba[idx + 2] = Math.round(rgba[idx + 2] * 0.65)
          }
        }
        if (cz > 0) {
          const hU = heights[(cz - 1) * hsx + cx] ?? h00
          if (Math.abs(h00 - hU) > 0.01 && (oy < 0.08 || oy > 0.92)) {
            rgba[idx] = Math.round(rgba[idx] * 0.65)
            rgba[idx + 1] = Math.round(rgba[idx + 1] * 0.65)
            rgba[idx + 2] = Math.round(rgba[idx + 2] * 0.65)
          }
        }
      } else {
        // Graded 模式：柔和边界，高度差>2 时按梯度加深
        if (cx > 0 && cz > 0) {
          const hL = heights[cz * hsx + (cx - 1)] ?? h00
          const hU = heights[(cz - 1) * hsx + cx] ?? h00
          const gX = Math.abs(h00 - hL)
          const gZ = Math.abs(h00 - hU)
          if (gX > 2 && (ox < 0.06 || ox > 0.94)) {
            const d = 1 - Math.min(1, gX * 0.015)
            rgba[idx] = Math.round(rgba[idx] * d)
            rgba[idx + 1] = Math.round(rgba[idx + 1] * d)
            rgba[idx + 2] = Math.round(rgba[idx + 2] * d)
          }
          if (gZ > 2 && (oy < 0.06 || oy > 0.94)) {
            const d = 1 - Math.min(1, gZ * 0.015)
            rgba[idx] = Math.round(rgba[idx] * d)
            rgba[idx + 1] = Math.round(rgba[idx + 1] * d)
            rgba[idx + 2] = Math.round(rgba[idx + 2] * d)
          }
        }
      }

      // 等高线（每 10 格一条）
      if (doContour) {
        const nearest10 = Math.round(h00 / contourInterval) * contourInterval
        const dist = Math.abs(h00 - nearest10)
        if (dist < 0.3) {
          const bright = Math.round((1 - dist / 0.3) * 40)
          rgba[idx] = Math.min(255, rgba[idx] + bright)
          rgba[idx + 1] = Math.min(255, rgba[idx + 1] + bright)
          rgba[idx + 2] = Math.min(255, rgba[idx + 2] + bright)
        }
      }
    }
  }
}
