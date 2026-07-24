/**
 * 种子地图坐标格式化与剪贴板工具
 *
 * 从 SeedMap.vue 拆出，避免 Vue 组件超 300 行。
 * 复用项目惯例：navigator.clipboard.writeText（见 ResourceDetailHeader.vue / ColorPalette.vue）
 */

/** 格式化方块坐标为 "X / Z" 形式（如 -1234 / 5678） */
export function formatCoord(x: number, z: number): string {
  return `${Math.round(x)} / ${Math.round(z)}`
}

/** 复制文本到剪贴板，成功返回 true */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    return false
  }
}
