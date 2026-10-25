/**
 * 配方工作台面板几何验收测试（对应 specs/cleanroom/recipe-generator.md §2 §6.8）
 */
import { describe, expect, it } from 'vitest'
import {
  RECIPE_IMAGE_HEIGHT,
  RECIPE_IMAGE_WIDTH,
  RECIPE_LAYOUTS,
  getRecipeLayout,
} from '../recipe-layouts'
import { RECIPE_TYPES } from '@/utils/recipe-generator/types'
import type { RecipeType } from '@/utils/recipe-generator/types'
import type { RecipeLayout } from '../recipe-layouts'

/** 取布局，缺失时直接失败（避免测试里到处判空） */
function layoutOf(type: RecipeType): RecipeLayout {
  const layout = getRecipeLayout(type)
  if (!layout) throw new Error(`缺少布局: ${type}`)
  return layout
}

describe('recipe-layouts', () => {
  it('should 覆盖全部配方类型 when 读取布局表', () => {
    for (const type of RECIPE_TYPES) {
      expect(getRecipeLayout(type)).toBeDefined()
    }
    expect(RECIPE_LAYOUTS).toHaveLength(RECIPE_TYPES.length)
  })

  it('should 由原点与步进推导 3×3 网格 when 查看合成面板', () => {
    const layout = layoutOf('crafting')
    const first = layout.slots['crafting.1']
    const second = layout.slots['crafting.2']
    const fourth = layout.slots['crafting.4']
    expect(first).toEqual({ x1: 117, y1: 64, x2: 181, y2: 128 })
    expect(second?.x1).toBe(117 + 71)
    expect(fourth?.y1).toBe(64 + 71)
    expect(layout.slots['crafting.9']).toEqual({ x1: 117 + 142, y1: 64 + 142, x2: 117 + 142 + 64, y2: 64 + 142 + 64 })
  })

  it('should 不越出面板边界 when 检查所有槽位盒', () => {
    for (const layout of RECIPE_LAYOUTS) {
      for (const [slot, box] of Object.entries(layout.slots)) {
        if (!box) continue
        expect(box.x1, `${layout.recipeType}/${slot} 左边界`).toBeGreaterThanOrEqual(0)
        expect(box.y1, `${layout.recipeType}/${slot} 上边界`).toBeGreaterThanOrEqual(0)
        expect(box.x2, `${layout.recipeType}/${slot} 右边界`).toBeLessThanOrEqual(RECIPE_IMAGE_WIDTH)
        expect(box.y2, `${layout.recipeType}/${slot} 下边界`).toBeLessThanOrEqual(RECIPE_IMAGE_HEIGHT)
      }
    }
  })

  it('should 互不重叠 when 检查同面板槽位', () => {
    for (const layout of RECIPE_LAYOUTS) {
      const boxes = Object.entries(layout.slots).filter(([, box]) => box) as [string, { x1: number; y1: number; x2: number; y2: number }][]
      for (let i = 0; i < boxes.length; i += 1) {
        for (let j = i + 1; j < boxes.length; j += 1) {
          const [slotA, a] = boxes[i]
          const [slotB, b] = boxes[j]
          const overlap = a.x1 < b.x2 && b.x1 < a.x2 && a.y1 < b.y2 && b.y1 < a.y2
          expect(overlap, `${layout.recipeType}: ${slotA} 与 ${slotB} 重叠`).toBe(false)
        }
      }
    }
  })
})
