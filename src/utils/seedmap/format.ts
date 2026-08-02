/**
 * 种子地图坐标格式化工具
 *
 * 从 SeedMap.vue 拆出，避免 Vue 组件超 300 行。
 * 剪贴板复制统一走 @/utils/clipboard。
 */
import { copyToClipboard } from '../clipboard'

export { copyToClipboard }

/** 格式化方块坐标为 "X / Z" 形式（如 -1234 / 5678） */
export function formatCoord(x: number, z: number): string {
  return `${Math.round(x)} / ${Math.round(z)}`
}
