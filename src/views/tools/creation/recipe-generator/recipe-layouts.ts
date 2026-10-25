/**
 * 配方工作台面板：槽位几何 + 工作台界面示意图
 *
 * 示意图是各工作台界面的截图，槽位盒与图上的凹槽一一对应；3×3 合成网格
 * 由「原点 + 步进」算出，其余面板按盒声明，新增配方类型只需补一条声明。
 */
import craftingImage from '@/assets/Syn/crafting.png?url'
import smeltingImage from '@/assets/Syn/smelting.png?url'
import campfireImage from '@/assets/Syn/campfire.png?url'
import stonecutterImage from '@/assets/Syn/stonecutter.png?url'
import smithingImage from '@/assets/Syn/smithing.png?url'
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

/** 单个槽位边长与 3×3 网格步进（槽位 64 + 间隔 7） */
const SLOT_SIZE = 64
const GRID_STRIDE = 71

function boxAt(x: number, y: number, width = SLOT_SIZE, height = width): RecipeLayoutSlotBox {
  return { x1: x, y1: y, x2: x + width, y2: y + height }
}

/** 3×3 合成网格：行优先编号 crafting.1 → crafting.9 */
function craftingGrid(
  originX: number,
  originY: number,
): Partial<Record<RecipeSlot, RecipeLayoutSlotBox>> {
  const slots: Partial<Record<RecipeSlot, RecipeLayoutSlotBox>> = {}
  for (let row = 0; row < 3; row += 1) {
    for (let column = 0; column < 3; column += 1) {
      const slot = `crafting.${row * 3 + column + 1}` as RecipeSlot
      slots[slot] = boxAt(originX + column * GRID_STRIDE, originY + row * GRID_STRIDE)
    }
  }
  return slots
}

type PanelSpec = {
  image: string
  slots: Partial<Record<RecipeSlot, RecipeLayoutSlotBox>>
}

/** 工作台：3×3 输入格 + 右侧产出 */
const CRAFTING: PanelSpec = {
  image: craftingImage,
  slots: { ...craftingGrid(117, 64), 'crafting.result': boxAt(476, 121, 95, 94) },
}

/** 熔炉系（熔炼 / 高炉 / 烟熏）共用一张界面：上投入、下燃料、右侧产出 */
const COOKING: PanelSpec = {
  image: smeltingImage,
  slots: {
    'cooking.ingredient': boxAt(220, 64, 63),
    'cooking.result': boxAt(444, 120, 95),
  },
}

const CAMPFIRE: PanelSpec = {
  image: campfireImage,
  slots: {
    'cooking.ingredient': boxAt(142, 132, 61, 63),
    'cooking.result': boxAt(512, 132, 63),
  },
}

const STONECUTTER: PanelSpec = {
  image: stonecutterImage,
  slots: {
    'stonecutter.ingredient': boxAt(76, 129, 63, 62),
    'stonecutter.result': boxAt(552, 112, 95),
  },
}

/** 锻造台：模板 / 底材 / 附加（修剪配方无产出位） */
const SMITHING: PanelSpec = {
  image: smithingImage,
  slots: {
    'smithing.template': boxAt(28, 188),
    'smithing.base': boxAt(99, 188),
    'smithing.addition': boxAt(170, 188),
  },
}

const SMITHING_WITH_RESULT: PanelSpec = {
  image: smithingImage,
  slots: { ...SMITHING.slots, 'smithing.result': boxAt(388, 188, 63) },
}

function layoutOf(recipeType: RecipeType, panel: PanelSpec): RecipeLayout {
  return { recipeType, image: panel.image, slots: panel.slots }
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
