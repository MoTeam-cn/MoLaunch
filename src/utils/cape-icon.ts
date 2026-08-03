/**
 * 披风图标提取工具
 *
 * 从完整披风正背面纹理图中裁剪外侧可见图案（10x16 区域）作为列表图标，
 * 支持高清纹理缩放，坐标依据 skinview3d CapeObject UV 布局，详见 getCapeIcon。
 */

import { loadImage, clipImageRegion } from '@/utils/image-crop'

/** 源裁剪区域（标准披风纹理坐标，正面可见图案区域） */
const CAPE_ICON_SRC_X = 1
const CAPE_ICON_SRC_Y = 1
const CAPE_ICON_SRC_W = 10
const CAPE_ICON_SRC_H = 16
/** 输出 canvas 尺寸（保留 10:16 宽高比） */
const CAPE_ICON_OUT_W = 10
const CAPE_ICON_OUT_H = 16

/**
 * 从披风 PNG URL 裁剪外侧可见面图标
 *
 * @param capeUrl 披风 PNG 的 URL（如 textures.minecraft.net 上的资源）
 * @returns data:image/png;base64,... 格式的图标 dataURL
 * @throws 图片加载失败或尺寸过小时抛出错误
 */
export async function getCapeIcon(capeUrl: string): Promise<string> {
  const img = await loadImage(capeUrl)
  const w = img.naturalWidth
  const h = img.naturalHeight
  if (w < 22 || h < 17) {
    throw new Error(`cape image too small: ${w}x${h}`)
  }
  // 支持高清披风：标准 64x32 (scale=1)，高清 128x64 (scale=2)
  const scale = w / 64
  const srcX = CAPE_ICON_SRC_X * scale
  const srcY = CAPE_ICON_SRC_Y * scale
  const srcW = CAPE_ICON_SRC_W * scale
  const srcH = CAPE_ICON_SRC_H * scale
  return clipImageRegion(img, srcX, srcY, srcW, srcH, CAPE_ICON_OUT_W, CAPE_ICON_OUT_H)
}
