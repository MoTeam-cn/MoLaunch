/**
 * 皮肤 UV 分区（Minecraft 公开格式）
 *
 * 现代 64×64 皮肤：6 个部件各有本体区与外层区（帽子/夹克/袖子/裤腿）；
 * 旧版 64×32 皮肤：无外层区，左肢与右肢共用同一块贴图。
 * 矩形坐标基于皮肤 PNG 像素坐标系，原点在左上角。
 */

/** 手臂模型：classic 4px 宽 / slim 3px 宽（仅影响预览，不影响贴图坐标） */
export type SkinVariant = 'classic' | 'slim'

/** 皮肤贴图布局：64×64 现代版 / 64×32 旧版 */
export type SkinLayout = 64 | 32

export type SkinPartId = 'head' | 'body' | 'rightArm' | 'leftArm' | 'rightLeg' | 'leftLeg'

export interface SkinRect {
  x: number
  y: number
  w: number
  h: number
}

export interface SkinPart {
  id: SkinPartId
  label: string
  /** 本体区（6 面展开） */
  base: SkinRect
  /** 外层区（帽子/外套等），旧版皮肤没有 */
  overlay?: SkinRect
  /** 旧版皮肤中该部件与另一侧共用贴图 */
  mirrored?: boolean
}

const MODERN_PARTS: SkinPart[] = [
  { id: 'head', label: '头部', base: { x: 0, y: 0, w: 32, h: 16 }, overlay: { x: 32, y: 0, w: 32, h: 16 } },
  { id: 'body', label: '身体', base: { x: 16, y: 16, w: 24, h: 16 }, overlay: { x: 16, y: 32, w: 24, h: 16 } },
  {
    id: 'rightArm',
    label: '右臂',
    base: { x: 40, y: 16, w: 16, h: 16 },
    overlay: { x: 40, y: 32, w: 16, h: 16 },
  },
  {
    id: 'leftArm',
    label: '左臂',
    base: { x: 32, y: 48, w: 16, h: 16 },
    overlay: { x: 48, y: 48, w: 16, h: 16 },
  },
  { id: 'rightLeg', label: '右腿', base: { x: 0, y: 16, w: 16, h: 16 }, overlay: { x: 0, y: 32, w: 16, h: 16 } },
  {
    id: 'leftLeg',
    label: '左腿',
    base: { x: 16, y: 48, w: 16, h: 16 },
    overlay: { x: 0, y: 48, w: 16, h: 16 },
  },
]

const LEGACY_PARTS: SkinPart[] = [
  { id: 'head', label: '头部', base: { x: 0, y: 0, w: 32, h: 16 } },
  { id: 'body', label: '身体', base: { x: 16, y: 16, w: 24, h: 16 } },
  { id: 'rightArm', label: '右臂', base: { x: 40, y: 16, w: 16, h: 16 } },
  { id: 'leftArm', label: '左臂', base: { x: 40, y: 16, w: 16, h: 16 }, mirrored: true },
  { id: 'rightLeg', label: '右腿', base: { x: 0, y: 16, w: 16, h: 16 } },
  { id: 'leftLeg', label: '左腿', base: { x: 0, y: 16, w: 16, h: 16 }, mirrored: true },
]

/** 按宽高识别皮肤布局；无法识别返回 null */
export function detectSkinLayout(width: number, height: number): SkinLayout | null {
  if (width === 64 && height === 64) return 64
  if (width === 64 && height === 32) return 32
  return null
}

/** 取指定布局的部件分区列表 */
export function getSkinParts(layout: SkinLayout): SkinPart[] {
  return layout === 32 ? LEGACY_PARTS : MODERN_PARTS
}

/** 点是否落在矩形内（左闭右开） */
export function rectContains(rect: SkinRect, x: number, y: number): boolean {
  return x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h
}

/** 命中最上层部件（外层区优先于本体区） */
export function findPartAt(parts: SkinPart[], x: number, y: number): SkinPart | undefined {
  return parts.find(
    (part) =>
      (part.overlay !== undefined && rectContains(part.overlay, x, y)) || rectContains(part.base, x, y),
  )
}

/** 布局对应的贴图边长（正方形） */
export function layoutSize(layout: SkinLayout): number {
  return layout
}
