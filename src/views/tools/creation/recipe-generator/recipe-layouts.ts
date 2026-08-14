/**
 * 配方槽位布局：背景图（各工作台 GUI 示意图）+ 槽位像素盒（696×292 坐标系）
 *
 * 背景图暂为临时占位（src/assets/Syn/*.png），后续自行替换
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

export const RECIPE_LAYOUTS: readonly RecipeLayout[] = [
  {
    recipeType: 'crafting',
    image: craftingImage,
    slots: {
      'crafting.1': { x1: 117, y1: 64, x2: 181, y2: 128 },
      'crafting.2': { x1: 188, y1: 64, x2: 252, y2: 128 },
      'crafting.3': { x1: 259, y1: 64, x2: 323, y2: 128 },
      'crafting.4': { x1: 117, y1: 135, x2: 181, y2: 199 },
      'crafting.5': { x1: 188, y1: 135, x2: 252, y2: 199 },
      'crafting.6': { x1: 259, y1: 135, x2: 323, y2: 199 },
      'crafting.7': { x1: 117, y1: 206, x2: 181, y2: 270 },
      'crafting.8': { x1: 188, y1: 206, x2: 252, y2: 270 },
      'crafting.9': { x1: 259, y1: 206, x2: 323, y2: 270 },
      'crafting.result': { x1: 476, y1: 121, x2: 571, y2: 215 },
    },
  },
  {
    recipeType: 'smelting',
    image: smeltingImage,
    slots: {
      'cooking.ingredient': { x1: 220, y1: 64, x2: 283, y2: 127 },
      'cooking.result': { x1: 444, y1: 120, x2: 539, y2: 215 },
    },
  },
  {
    recipeType: 'blasting',
    image: smeltingImage,
    slots: {
      'cooking.ingredient': { x1: 220, y1: 64, x2: 283, y2: 127 },
      'cooking.result': { x1: 444, y1: 120, x2: 539, y2: 215 },
    },
  },
  {
    recipeType: 'smoking',
    image: smeltingImage,
    slots: {
      'cooking.ingredient': { x1: 220, y1: 64, x2: 283, y2: 127 },
      'cooking.result': { x1: 444, y1: 120, x2: 539, y2: 215 },
    },
  },
  {
    recipeType: 'campfire_cooking',
    image: campfireImage,
    slots: {
      'cooking.ingredient': { x1: 142, y1: 132, x2: 203, y2: 195 },
      'cooking.result': { x1: 512, y1: 132, x2: 575, y2: 195 },
    },
  },
  {
    recipeType: 'stonecutter',
    image: stonecutterImage,
    slots: {
      'stonecutter.ingredient': { x1: 76, y1: 129, x2: 139, y2: 191 },
      'stonecutter.result': { x1: 552, y1: 112, x2: 647, y2: 207 },
    },
  },
  {
    recipeType: 'smithing',
    image: smithingImage,
    slots: {
      'smithing.template': { x1: 28, y1: 188, x2: 92, y2: 252 },
      'smithing.base': { x1: 99, y1: 188, x2: 163, y2: 252 },
      'smithing.addition': { x1: 170, y1: 188, x2: 234, y2: 252 },
      'smithing.result': { x1: 388, y1: 188, x2: 451, y2: 251 },
    },
  },
  {
    recipeType: 'smithing_trim',
    image: smithingImage,
    slots: {
      'smithing.template': { x1: 28, y1: 188, x2: 92, y2: 252 },
      'smithing.base': { x1: 99, y1: 188, x2: 163, y2: 252 },
      'smithing.addition': { x1: 170, y1: 188, x2: 234, y2: 252 },
    },
  },
  {
    recipeType: 'smithing_transform',
    image: smithingImage,
    slots: {
      'smithing.template': { x1: 28, y1: 188, x2: 92, y2: 252 },
      'smithing.base': { x1: 99, y1: 188, x2: 163, y2: 252 },
      'smithing.addition': { x1: 170, y1: 188, x2: 234, y2: 252 },
      'smithing.result': { x1: 388, y1: 188, x2: 451, y2: 251 },
    },
  },
]

export function getRecipeLayout(recipeType: RecipeType): RecipeLayout | null {
  return RECIPE_LAYOUTS.find((layout) => layout.recipeType === recipeType) ?? null
}
