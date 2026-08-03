/**
 * 图片加载与裁剪工具
 * 提供通用 Image 加载 + canvas 裁剪，供 cape-icon.ts、SkinAvatar.vue 等复用。
 * 统一 crossOrigin、imageSmoothingEnabled=false（保持像素风格）、toDataURL 输出。
 */

/**
 * 加载图片为 HTMLImageElement
 *
 * @param url 图片 URL
 * @returns 加载完成的 HTMLImageElement
 * @throws 图片加载失败时抛出错误
 */
export function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.crossOrigin = 'anonymous'
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error(`load image failed: ${url}`))
    img.src = url
  })
}

/**
 * 裁剪图片指定区域并返回 dataURL
 *
 * @param img 已加载的 HTMLImageElement
 * @param sx 源 x 坐标
 * @param sy 源 y 坐标
 * @param sw 源宽度
 * @param sh 源高度
 * @param outW 输出宽度（默认等于 sw）
 * @param outH 输出高度（默认等于 sh）
 * @param willReadFrequently 是否需要频繁读取像素（透明度检测等场景设为 true）
 * @returns data:image/png;base64,... 格式的 dataURL
 */
export function clipImageRegion(
  img: HTMLImageElement,
  sx: number,
  sy: number,
  sw: number,
  sh: number,
  outW?: number,
  outH?: number,
  willReadFrequently?: boolean,
): string {
  const w = outW ?? sw
  const h = outH ?? sh
  const canvas = document.createElement('canvas')
  canvas.width = w
  canvas.height = h
  const ctx = canvas.getContext('2d', willReadFrequently ? { willReadFrequently: true } : undefined)
  if (!ctx) {
    throw new Error('canvas 2d context not available')
  }
  ctx.imageSmoothingEnabled = false
  ctx.drawImage(img, sx, sy, sw, sh, 0, 0, w, h)
  return canvas.toDataURL('image/png')
}
