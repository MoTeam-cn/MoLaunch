/**
 * 投影页面共享类型
 *
 * 工具种类与画布交互模式的映射集中在这里，避免各组件各自定义导致不一致。
 */

/** 编辑工具 */
export type SchematicTool = 'browse' | 'pan' | 'brush' | 'rect' | 'sphere' | 'cylinder' | 'replace'

/** 画布交互模式 */
export type CanvasInteraction = 'browse' | 'pan' | 'brush' | 'rect' | 'point'

/** 工具 → 画布交互模式 */
export const TOOL_INTERACTION: Record<SchematicTool, CanvasInteraction> = {
  browse: 'browse',
  pan: 'pan',
  brush: 'brush',
  rect: 'rect',
  replace: 'rect',
  sphere: 'point',
  cylinder: 'point',
}

/** 工具是否需要先选方块 */
export const EDIT_TOOLS: SchematicTool[] = ['brush', 'rect', 'sphere', 'cylinder', 'replace']
