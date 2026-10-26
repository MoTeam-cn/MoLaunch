/**
 * 皮肤 PNG 与像素缓冲互转（依赖浏览器 canvas）
 */

import { PixelBuffer } from './pixel-buffer'
import { detectSkinLayout, type SkinLayout } from './layout'

export interface DecodedSkin {
  buffer: PixelBuffer
  layout: SkinLayout
}

/** 解码 PNG dataURL 为像素缓冲；尺寸不合法时抛错 */
export async function decodeSkinPng(dataUrl: string): Promise<DecodedSkin> {
  const image = await loadImage(dataUrl)
  const layout = detectSkinLayout(image.naturalWidth, image.naturalHeight)
  if (!layout) {
    throw new Error(`皮肤尺寸需为 64×64 或 64×32，当前为 ${image.naturalWidth}×${image.naturalHeight}`)
  }
  const canvas = document.createElement('canvas')
  canvas.width = image.naturalWidth
  canvas.height = image.naturalHeight
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('无法创建画布上下文')
  ctx.drawImage(image, 0, 0)
  const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data
  return { buffer: new PixelBuffer(canvas.width, canvas.height, data), layout }
}

/** 像素缓冲编码为 PNG dataURL */
export function encodeSkinPng(buffer: PixelBuffer): string {
  const canvas = document.createElement('canvas')
  canvas.width = buffer.width
  canvas.height = buffer.height
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('无法创建画布上下文')
  const imageData = ctx.createImageData(buffer.width, buffer.height)
  imageData.data.set(buffer.toRgba())
  ctx.putImageData(imageData, 0, 0)
  return canvas.toDataURL('image/png')
}

/** 把像素缓冲画到目标 canvas（像素风：关闭平滑） */
export function drawBufferToCanvas(
  target: HTMLCanvasElement,
  buffer: PixelBuffer,
  zoom: number,
): void {
  const ctx = target.getContext('2d')
  if (!ctx) return
  const source = document.createElement('canvas')
  source.width = buffer.width
  source.height = buffer.height
  const sourceCtx = source.getContext('2d')
  if (!sourceCtx) return
  const imageData = sourceCtx.createImageData(buffer.width, buffer.height)
  imageData.data.set(buffer.toRgba())
  sourceCtx.putImageData(imageData, 0, 0)

  target.width = buffer.width * zoom
  target.height = buffer.height * zoom
  ctx.imageSmoothingEnabled = false
  ctx.clearRect(0, 0, target.width, target.height)
  ctx.drawImage(source, 0, 0, target.width, target.height)
}

function loadImage(dataUrl: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image()
    image.onload = () => resolve(image)
    image.onerror = () => reject(new Error('图片解码失败'))
    image.src = dataUrl
  })
}
