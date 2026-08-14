/**
 * 成就生成器 - Canvas 绘制
 *
 * 320×64 逻辑尺寸、2 倍率绘制保证清晰；三层直角边框模拟原版成就弹窗，
 * 18px 粗像素字与参考图质感一致，图标加深色衬底，右下角叠加白色 MoLaunch 版权水印。
 */

export const ACHIEVEMENT_SIZE = { width: 320, height: 64 } as const
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

/** 绘制成就弹窗（内部按逻辑尺寸绘制，画布像素尺寸由调用方按 ACHIEVEMENT_SCALE 设置） */
export function drawAchievement(ctx: CanvasRenderingContext2D, opts: AchievementOptions) {
  const { width, height } = ACHIEVEMENT_SIZE
  ctx.setTransform(ACHIEVEMENT_SCALE, 0, 0, ACHIEVEMENT_SCALE, 0, 0)
  ctx.clearRect(0, 0, width, height)
  ctx.imageSmoothingEnabled = false

  // 三层直角边框：外黑 2px + 中灰 4px + 内深灰底（与原版成就弹窗一致，无圆角）
  ctx.fillStyle = '#010101'
  ctx.fillRect(0, 0, width, height)
  ctx.fillStyle = '#555555'
  ctx.fillRect(2, 2, width - 4, height - 4)
  ctx.fillStyle = '#212121'
  ctx.fillRect(6, 6, width - 12, height - 12)

  // 物品图标 30×30，垂直居中于左侧；外围 1px 深色衬底提升层次感
  const iconX = 17
  if (opts.icon) {
    const [sx, sy, sw, sh] = opts.icon.region
    const iconY = (height - 30) / 2
    ctx.fillStyle = '#141414'
    ctx.fillRect(iconX - 1, iconY - 1, 32, 32)
    ctx.drawImage(opts.icon.img, sx, sy, sw, sh, iconX, iconY, 30, 30)
  }

  // 标题 / 内容两行文字：18px 粗像素字（字形高约 12px，与参考图一致），纯色无描边
  const textX = 60
  const textMaxWidth = width - textX - 14
  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.font = `bold 18px ${opts.fontFamily}`
  ctx.fillStyle = opts.titleColor
  ctx.fillText(truncateText(ctx, opts.title, textMaxWidth), textX, 28)
  ctx.fillStyle = opts.contentColor
  ctx.fillText(truncateText(ctx, opts.content, textMaxWidth), textX, 50)

  // MoLaunch 版权水印（右下角，小字半透明，不抢主体）
  ctx.font = `8px ${opts.fontFamily}`
  ctx.fillStyle = 'rgba(255, 255, 255, 0.5)'
  ctx.textAlign = 'right'
  ctx.fillText('MoLaunch', width - 6, height - 6)
  ctx.textAlign = 'left'
}
