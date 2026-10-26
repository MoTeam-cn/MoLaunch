/**
 * 工具模块 - 投影（schematic）会话式预览、编辑与导出
 *
 * 对应后端 tools_manager 的 schematic_open / slice / edit / history / transform /
 * export / close action。打开后数据留在后端内存，前端按视口取切片。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

export interface SchematicBlockCount {
  id: string
  count: number
}

export interface SchematicOpenResult {
  sessionId: string
  format: string
  name: string
  size: [number, number, number]
  palette: string[]
  total: number
  nonAir: number
  counts: SchematicBlockCount[]
}

export interface SchematicOverviewResult {
  size: [number, number, number]
  layerCounts: number[]
  width: number
  height: number
  step: number
  topIndexBase64: string
  topHeightBase64: string
  emptyIndex: number
}

export interface SchematicSliceResult {
  x: number
  y: number
  width: number
  height: number
  cellsBase64: string
}

export interface SchematicEditResult {
  changed: number
  nonAir: number
  canUndo: boolean
  canRedo: boolean
}

export interface SchematicTransformResult {
  size: [number, number, number]
  nonAir: number
}

export interface SchematicExportResult {
  path: string
  bytes: number
}

export type SliceAxisParam = 'x' | 'y' | 'z'

export async function schematicOpen(path: string): Promise<SchematicOpenResult> {
  return toolsManager<SchematicOpenResult>(TOOLS_ACTIONS.SCHEMATIC_OPEN, { path })
}

export async function schematicOverview(
  sessionId: string,
): Promise<SchematicOverviewResult> {
  return toolsManager<SchematicOverviewResult>(TOOLS_ACTIONS.SCHEMATIC_OVERVIEW, { sessionId })
}

export async function schematicSlice(
  sessionId: string,
  axis: SliceAxisParam,
  layer: number,
  x: number,
  y: number,
  w: number,
  h: number,
): Promise<SchematicSliceResult> {
  return toolsManager<SchematicSliceResult>(TOOLS_ACTIONS.SCHEMATIC_SLICE, {
    sessionId,
    axis,
    layer,
    x,
    y,
    w,
    h,
  })
}

export type EditShape = 'rect' | 'sphere' | 'cylinder' | 'points'

export interface SchematicEditParams {
  sessionId: string
  axis: SliceAxisParam
  layer: number
  x: number
  y: number
  w: number
  h: number
  block: number
  mode: 'fill' | 'replace'
  from?: number | null
  /** 选区形状：矩形 / 球体 / 圆柱 / 画笔落点 */
  shape?: EditShape
  /** 球体与圆柱半径 */
  radius?: number
  /** 圆柱沿切片轴高度 */
  height?: number
  /** 画笔落点（平面坐标） */
  points?: Array<[number, number]>
}

export async function schematicEdit(params: SchematicEditParams): Promise<SchematicEditResult> {
  return toolsManager<SchematicEditResult>(TOOLS_ACTIONS.SCHEMATIC_EDIT, params)
}

export async function schematicHistory(
  sessionId: string,
  action: 'undo' | 'redo',
): Promise<SchematicEditResult> {
  return toolsManager<SchematicEditResult>(TOOLS_ACTIONS.SCHEMATIC_HISTORY, { sessionId, action })
}

export async function schematicTransform(
  sessionId: string,
  kind: 'rotate_cw' | 'rotate_ccw' | 'mirror_x' | 'mirror_z' | 'flip_y',
): Promise<SchematicTransformResult> {
  return toolsManager<SchematicTransformResult>(TOOLS_ACTIONS.SCHEMATIC_TRANSFORM, { sessionId, kind })
}

export async function schematicExport(
  sessionId: string,
  targetPath: string,
  format: string,
): Promise<SchematicExportResult> {
  return toolsManager<SchematicExportResult>(TOOLS_ACTIONS.SCHEMATIC_EXPORT, {
    sessionId,
    targetPath,
    format,
  })
}

export async function schematicClose(sessionId: string): Promise<void> {
  await toolsManager(TOOLS_ACTIONS.SCHEMATIC_CLOSE, { sessionId })
}
