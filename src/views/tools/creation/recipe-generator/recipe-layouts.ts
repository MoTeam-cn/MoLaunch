/**
 * 配方工作台面板：槽位几何 + 面板示意图
 *
 * 面板示意图由槽位几何在运行时渲染成 SVG（data URL），因此示意图与可点击
 * 热点天然对齐，也不依赖任何外部图片资源；新增配方类型只需声明几何。
 */
import type { RecipeSlot, RecipeType } from '@/utils/recipe-generator/types'

export const RECIPE_IMAGE_WIDTH = 696
export const RECIPE_IMAGE_HEIGHT = 292

export type RecipeLayoutSlotBox = {
  x1: number
  y1: number
  x2: number
  y2: number
}

export type RecipeLayout = {
  recipeType: RecipeType
  image: string
  slots: Partial<Record<RecipeSlot, RecipeLayoutSlotBox>>
}

/** 单个槽位边长与 3×3 网格步进 */
const SLOT_SIZE = 64
const GRID_STRIDE = 71

/** 面板配色（原版工作台界面观感，深浅主题下都可读） */
const PANEL = {
  body: '#C6C6C6',
  edge: '#3F3F3F',
  bevelLight: '#FFFFFF',
  bevelDark: '#555555',
  slotFill: '#8B8B8B',
  slotShadow: '#373737',
  arrow: '#8B8B8B',
} as const

type PanelSpec = {
  /** 可交互槽位 */
  slots: Partial<Record<RecipeSlot, RecipeLayoutSlotBox>>
  /** 仅作示意、不响应点击的凹槽（如熔炉燃料位） */
  decorative?: RecipeLayoutSlotBox[]
  /** 输入 → 输出箭头 */
  arrows?: { from: number; to: number; y: number }[]
}

function boxAt(x: number, y: number, width = SLOT_SIZE, height = width): RecipeLayoutSlotBox {
  return { x1: x, y1: y, x2: x + width, y2: y + height }
}

/** 3×3 合成网格：行优先编号 crafting.1 → crafting.9 */
function craftingGrid(originX: number, originY: number): Partial<Record<RecipeSlot, RecipeLayoutSlotBox>> {
  const slots: Partial<Record<RecipeSlot, RecipeLayoutSlotBox>> = {}
  for (let row = 0; row < 3; row += 1) {
    for (let column = 0; column < 3; column += 1) {
      const slot = `crafting.${row * 3 + column + 1}` as RecipeSlot
      slots[slot] = boxAt(originX + column * GRID_STRIDE, originY + row * GRID_STRIDE)
    }
  }
  return slots
}

function drawSlot(box: RecipeLayoutSlotBox): string {
  const width = box.x2 - box.x1
  const height = box.y2 - box.y1
  return [
    `<rect x="${box.x1}" y="${box.y1}" width="${width}" height="${height}" fill="${PANEL.slotFill}"/>`,
    `<path d="M ${box.x1} ${box.y2} L ${box.x1} ${box.y1} L ${box.x2} ${box.y1}" fill="none" stroke="${PANEL.slotShadow}" stroke-width="3"/>`,
    `<path d="M ${box.x2 - 2} ${box.y1 + 2} L ${box.x2 - 2} ${box.y2 - 2} L ${box.x1 + 2} ${box.y2 - 2}" fill="none" stroke="${PANEL.bevelLight}" stroke-width="2"/>`,
  ].join('')
}

function drawArrow(from: number, to: number, y: number): string {
  const head = 24
  const shaftEnd = Math.max(from, to - head)
  return [
    `<rect x="${from}" y="${y - 7}" width="${Math.max(0, shaftEnd - from)}" height="14" fill="${PANEL.arrow}"/>`,
    `<path d="M ${shaftEnd} ${y - 20} L ${to} ${y} L ${shaftEnd} ${y + 20} Z" fill="${PANEL.arrow}"/>`,
  ].join('')
}

function panelMarkup(spec: PanelSpec): string {
  const body = [
    `<rect x="4" y="4" width="${RECIPE_IMAGE_WIDTH - 8}" height="${RECIPE_IMAGE_HEIGHT - 8}" rx="10" fill="${PANEL.body}" stroke="${PANEL.edge}" stroke-width="4"/>`,
    `<path d="M 12 ${RECIPE_IMAGE_HEIGHT - 12} L 12 12 L ${RECIPE_IMAGE_WIDTH - 12} 12" fill="none" stroke="${PANEL.bevelLight}" stroke-width="3" opacity="0.65"/>`,
    `<path d="M ${RECIPE_IMAGE_WIDTH - 12} 12 L ${RECIPE_IMAGE_WIDTH - 12} ${RECIPE_IMAGE_HEIGHT - 12} L 12 ${RECIPE_IMAGE_HEIGHT - 12}" fill="none" stroke="${PANEL.bevelDark}" stroke-width="3" opacity="0.55"/>`,
  ]
  const slots = [...Object.values(spec.slots), ...(spec.decorative ?? [])]
    .filter((box): box is RecipeLayoutSlotBox => Boolean(box))
    .map(drawSlot)
  const arrows = (spec.arrows ?? []).map((arrow) => drawArrow(arrow.from, arrow.to, arrow.y))
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${RECIPE_IMAGE_WIDTH} ${RECIPE_IMAGE_HEIGHT}" width="${RECIPE_IMAGE_WIDTH}" height="${RECIPE_IMAGE_HEIGHT}">${[...body, ...slots, ...arrows].join('')}</svg>`
}

/** 面板示意图（data URL，可直接作为 CSS 背景） */
function panelImage(spec: PanelSpec): string {
  return `data:image/svg+xml,${encodeURIComponent(panelMarkup(spec))}`
}

const CRAFTING: PanelSpec = {
  slots: { ...craftingGrid(117, 64), 'crafting.result': boxAt(476, 121, 95) },
  arrows: [{ from: 352, to: 452, y: 168 }],
}

/** 熔炉系（熔炼 / 高炉 / 烟熏）共用一张面板：上投入、下燃料、右侧产出 */
const COOKING: PanelSpec = {
  slots: {
    'cooking.ingredient': boxAt(220, 64, 63),
    'cooking.result': boxAt(444, 120, 95),
  },
  decorative: [boxAt(220, 160, 63), boxAt(444, 246, 95, 40)],
  arrows: [{ from: 300, to: 430, y: 168 }],
}

const CAMPFIRE: PanelSpec = {
  slots: {
    'cooking.ingredient': boxAt(142, 132, 61, 63),
    'cooking.result': boxAt(512, 132, 63),
  },
  arrows: [{ from: 330, to: 496, y: 163 }],
}

const STONECUTTER: PanelSpec = {
  slots: {
    'stonecutter.ingredient': boxAt(76, 129, 63, 62),
    'stonecutter.result': boxAt(552, 112, 95),
  },
  arrows: [{ from: 250, to: 536, y: 160 }],
}

/** 锻造台：模板 / 底材 / 附加，右侧产出（修剪配方无产出位） */
const SMITHING: PanelSpec = {
  slots: {
    'smithing.template': boxAt(28, 188),
    'smithing.base': boxAt(99, 188),
    'smithing.addition': boxAt(170, 188),
  },
  arrows: [{ from: 250, to: 370, y: 220 }],
}

const SMITHING_WITH_RESULT: PanelSpec = {
  ...SMITHING,
  slots: { ...SMITHING.slots, 'smithing.result': boxAt(388, 188, 63) },
}

function layoutOf(recipeType: RecipeType, spec: PanelSpec): RecipeLayout {
  return { recipeType, image: panelImage(spec), slots: spec.slots }
}

export const RECIPE_LAYOUTS: readonly RecipeLayout[] = [
  layoutOf('crafting', CRAFTING),
  layoutOf('smelting', COOKING),
  layoutOf('blasting', COOKING),
  layoutOf('smoking', COOKING),
  layoutOf('campfire_cooking', CAMPFIRE),
  layoutOf('stonecutter', STONECUTTER),
  layoutOf('smithing', SMITHING_WITH_RESULT),
  layoutOf('smithing_trim', SMITHING),
  layoutOf('smithing_transform', SMITHING_WITH_RESULT),
]

export function getRecipeLayout(recipeType: RecipeType): RecipeLayout | null {
  return RECIPE_LAYOUTS.find((layout) => layout.recipeType === recipeType) ?? null
}
