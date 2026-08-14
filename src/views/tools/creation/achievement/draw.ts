/**
 * 成就生成器 - Canvas 绘制
 *
 * 320×65 逻辑尺寸、2 倍率绘制保证清晰；三层边框模拟原版成就弹窗，
 * 右下角固定叠加白色 MoLaunch 版权水印（不可关闭）。
 */

export const ACHIEVEMENT_SIZE = { width: 320, height: 65 } as const
export const ACHIEVEMENT_SCALE = 2

export interface AchievementOptions {
  title: string
  titleColor: string
  content: string
  contentColor: string
  /** Canvas font 的 font-family 片段（如 `Consolas, 'Courier New', monospace`） */
  fontFamily: string
  /** 物品图标：纹理图集图片 + 图集内裁剪区域 [x, y, w, h] */
  icon: { img: HTMLImageElement; region: [number, number, number, number] } | null
}

/** 按最大宽度截断文本，超出部分以省略号结尾（逐字裁剪，保证中文不断字） */
function truncateText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string {
  if (ctx.measureText(text).width <= maxWidth) return text
  let result = text
  while (result.length > 0 && ctx.measureText(`${result}…`).width > maxWidth) {
    result = result.slice(0, -1)
  }
  return result ? `${result}…` : ''
}

/** 圆角矩形路径（原版成就弹窗圆角约 5px） */
function roundedRectPath(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
) {
  ctx.beginPath()
  ctx.moveTo(x + r, y)
  ctx.arcTo(x + w, y, x + w, y + h, r)
  ctx.arcTo(x + w, y + h, x, y + h, r)
  ctx.arcTo(x, y + h, x, y, r)
  ctx.arcTo(x, y, x + w, y, r)
  ctx.closePath()
}

/** 绘制成就弹窗（内部按逻辑尺寸绘制，画布像素尺寸由调用方按 ACHIEVEMENT_SCALE 设置） */
export function drawAchievement(ctx: CanvasRenderingContext2D, opts: AchievementOptions) {
  const { width, height } = ACHIEVEMENT_SIZE
  ctx.setTransform(ACHIEVEMENT_SCALE, 0, 0, ACHIEVEMENT_SCALE, 0, 0)
  ctx.clearRect(0, 0, width, height)
  ctx.imageSmoothingEnabled = false

  // 三层边框：外黑 2px + 中灰 4px + 内深灰底
  roundedRectPath(ctx, 0, 0, width, height, 5)
  ctx.fillStyle = '#000000'
  ctx.fill()
  roundedRectPath(ctx, 2, 2, width - 4, height - 4, 5)
  ctx.fillStyle = '#555555'
  ctx.fill()
  roundedRectPath(ctx, 6, 6, width - 12, height - 12, 4)
  ctx.fillStyle = '#212121'
  ctx.fill()

  // 物品图标 30×30，垂直居中于左侧
  if (opts.icon) {
    const [sx, sy, sw, sh] = opts.icon.region
    ctx.drawImage(opts.icon.img, sx, sy, sw, sh, 10, (height - 30) / 2, 30, 30)
  }

  // 标题 / 内容两行文字
  const textX = 48
  const textMaxWidth = width - textX - 14
  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.font = `bold 12px ${opts.fontFamily}`
  ctx.fillStyle = opts.titleColor
  ctx.fillText(truncateText(ctx, opts.title, textMaxWidth), textX, 27)
  ctx.font = `bold 11px ${opts.fontFamily}`
  ctx.fillStyle = opts.contentColor
  ctx.fillText(truncateText(ctx, opts.content, textMaxWidth), textX, 45)

  // MoLaunch 版权水印（右下角，白色 0.85 不透明度）
  ctx.font = `9px ${opts.fontFamily}`
  ctx.fillStyle = 'rgba(255, 255, 255, 0.85)'
  ctx.textAlign = 'right'
  ctx.fillText('MoLaunch', width - 5, height - 4)
  ctx.textAlign = 'left'
}
