/**
 * 披风图标提取工具
 *
 * Minecraft 服务器返回的披风 PNG 是完整的正背面纹理图（标准 64x32，
 * 高清版 128x64 等），直接用作列表图标会显示过多内容。
 *
 * 根据 skinview3d 的 CapeObject 实现（node_modules/skinview3d/libs/model.js）：
 *   setCapeUVs(capeBox, 0, 0, 10, 16, 1)  // 纹理尺寸 64x32
 * 披风盒子为 10x16x1，UV 起点为 (0,0)，6 个面的 UV 区域如下：
 *   - top    (1,0)→(11,1)    顶部边缘
 *   - bottom (11,0)→(21,1)   底部边缘
 *   - left   (0,1)→(1,17)    左侧边缘
 *   - front  (1,1)→(11,17)   披风外侧可见图案（玩家看到的一面）
 *   - right  (11,1)→(12,17)  右侧边缘
 *   - back   (12,1)→(22,17)  披风内侧（贴着玩家后背）
 *
 * 本工具裁剪 front 面（外侧可见图案）的 10x16 区域作为图标。
 * 注：skinview3d 源码注释将几何面命名为 front/back，但实际纹理图上
 * (1,1)→(11,17) 区域才是玩家可见的披风图案。
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
