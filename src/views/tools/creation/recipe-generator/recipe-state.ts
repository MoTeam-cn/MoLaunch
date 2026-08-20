/**
 * 合成配方生成器 - 页面级状态工厂与静态选项
 */
import type { JavaVersionId, RecipeState } from '@/utils/recipe-generator/types'

let uidCounter = 0
export function createUid(prefix: string): string {
  uidCounter += 1
  return `${prefix}-${Date.now().toString(36)}-${uidCounter}`
}

export function createDefaultRecipe(): RecipeState {
  return {
    id: createUid('recipe'),
    recipeType: 'crafting',
    group: '',
    category: 'misc',
    showNotification: true,
    nameMode: 'manual',
    name: 'my_recipe',
    slots: {},
    crafting: { shapeless: false, keepWhitespace: false, twoByTwo: false },
    cooking: { time: null, experience: 0 },
    smithing: { trimPattern: '' },
  }
}

export const VERSION_OPTIONS: { label: string; value: JavaVersionId }[] = [
  { label: '1.12', value: '1.12' },
  { label: '1.13', value: '1.13' },
  { label: '1.14', value: '1.14' },
  { label: '1.15', value: '1.15' },
  { label: '1.16', value: '1.16' },
  { label: '1.17', value: '1.17' },
  { label: '1.18', value: '1.18' },
  { label: '1.19', value: '1.19' },
  { label: '1.20', value: '1.20' },
  { label: '1.21', value: '1.21' },
  { label: '1.21.2', value: '1.21.2' },
  { label: '1.21.4', value: '1.21.4' },
  { label: '1.21.5', value: '1.21.5' },
  { label: '1.21.6', value: '1.21.6' },
  { label: '1.21.7', value: '1.21.7' },
  { label: '1.21.9', value: '1.21.9' },
  { label: '1.21.11', value: '1.21.11' },
  { label: '26.1', value: '26.1' },
  { label: '26.2', value: '26.2' },
]

export const PALETTE_TAB_OPTIONS = [
  { label: '物品', value: 'items' },
  { label: '标签', value: 'tags' },
]