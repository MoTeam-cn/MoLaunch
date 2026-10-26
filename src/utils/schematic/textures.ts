/**
 * 投影方块贴图
 *
 * 复用配方生成器的纹理图集（16×16 原版贴图紧密排列），让逐层视图与等轴测预览
 * 显示真实方块纹理，而不是纯色块。取不到贴图时返回 null，由调用方退回纯色。
 */
import { getAtlasLayout, getAtlasPngUrl, loadVersionItems } from '@/utils/recipe-generator/resources'
import { LATEST_JAVA_VERSION } from '@/utils/recipe-generator/versions'

/** 图集内一块贴图的矩形 [x, y, w, h] */
export type TextureRect = [number, number, number, number]

export interface BlockAtlas {
  image: HTMLImageElement
  /** 图集 PNG 的地址，供 CSS background 使用 */
  url: string
  size: [number, number]
  layout: Record<string, TextureRect>
  /** 方块名（无命名空间）→ 图集 key，来自物品表，比按名字猜更准 */
  itemTextures: Map<string, string>
}

let cached: Promise<BlockAtlas | null> | null = null

/** 加载方块贴图图集；失败返回 null（纯色兜底），不会抛错 */
export function loadBlockAtlas(): Promise<BlockAtlas | null> {
  cached ??= (async () => {
    try {
      const [layout, items] = await Promise.all([
        getAtlasLayout(),
        loadVersionItems(LATEST_JAVA_VERSION).catch(() => []),
      ])
      if (!layout.size[0] || !Object.keys(layout.layout).length) return null
      const url = getAtlasPngUrl()
      const image = await loadImage(url)
      const itemTextures = new Map<string, string>()
      for (const item of items) {
        if (item.texture) itemTextures.set(stripNamespace(item.id), item.texture)
      }
      return { image, url, size: layout.size, layout: layout.layout, itemTextures }
    } catch {
      return null
    }
  })()
  return cached
}

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image()
    image.onload = () => resolve(image)
    image.onerror = () => reject(new Error('贴图图集加载失败'))
    image.src = url
  })
}

/** 去掉命名空间与 blockstate 属性 */
function stripNamespace(blockId: string): string {
  return blockId.replace(/^minecraft:/, '').split('[')[0]
}

/** 半砖、楼梯、墙上的告示牌等没有独立物品贴图，按材料回退 */
function aliasKeys(name: string): string[] {
  const out: string[] = []
  if (name === 'petrified_oak_slab') out.push('blocks/oak_planks')
  const wallSign = /^(.+)_wall_sign$/.exec(name)
  if (wallSign) out.push(`items/${wallSign[1]}_sign`, 'items/oak_sign')
  if (name === 'wall_torch') out.push('blocks/torch')
  const wallTorch = /^(.+)_wall_torch$/.exec(name)
  if (wallTorch) out.push(`blocks/${wallTorch[1]}_torch`)
  const slab = /^(.+)_slab$/.exec(name)
  if (slab) out.push(`blocks/${slab[1]}s`, `blocks/${slab[1]}`, `blocks/${slab[1]}_planks`)
  const stairs = /^(.+)_stairs$/.exec(name)
  if (stairs) out.push(`blocks/${stairs[1]}_planks`, `blocks/${stairs[1]}s`, `blocks/${stairs[1]}`)
  const carpet = /^(.+)_carpet$/.exec(name)
  if (carpet) out.push(`blocks/${carpet[1]}_wool`)
  return out
}

/** 方块 id → 图集贴图矩形；没有对应贴图返回 null */
export function resolveTexture(atlas: BlockAtlas | null, blockId: string): TextureRect | null {
  if (!atlas) return null
  const name = stripNamespace(blockId)
  if (!name || name === 'air') return null
  const keys: string[] = []
  const fromItem = atlas.itemTextures.get(name)
  if (fromItem) keys.push(fromItem)
  keys.push(`blocks/${name}`, `items/${name}`)
  keys.push(...aliasKeys(name))
  for (const key of keys) {
    const rect = atlas.layout[key]
    if (rect) return rect
  }
  return null
}

/** 以像素风绘制整块贴图到指定矩形 */
export function drawTexture(
  ctx: CanvasRenderingContext2D,
  atlas: BlockAtlas,
  rect: TextureRect,
  dx: number,
  dy: number,
  dw: number,
  dh: number,
): void {
  ctx.drawImage(atlas.image, rect[0], rect[1], rect[2], rect[3], dx, dy, dw, dh)
}
