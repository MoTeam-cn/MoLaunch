/**
 * 工具模块 - 投影（schematic）预览与导出
 *
 * 对应后端 tools_manager 的 schematic_open / schematic_export action。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

/** 方块统计条目 */
export interface SchematicBlockCount {
  id: string
  count: number
}

/** 打开投影的结果 */
export interface SchematicOpenResult {
  /** sponge_v1 / sponge_v2 / litematic / structure */
  format: string
  name: string
  /** [x, y, z] */
  size: [number, number, number]
  /** 索引 -> 方块 id */
  palette: string[]
  /** u16 小端数组的 base64，长度 = x*y*z */
  blocksBase64: string
  total: number
  nonAir: number
  counts: SchematicBlockCount[]
}

/** 导出结果 */
export interface SchematicExportResult {
  path: string
  bytes: number
}

/** 打开投影文件 */
export async function schematicOpen(path: string): Promise<SchematicOpenResult> {
  return toolsManager<SchematicOpenResult>(TOOLS_ACTIONS.SCHEMATIC_OPEN, { path })
}

/** 导出投影文件（format: sponge_v2 / litematic / structure） */
export async function schematicExport(
  sourcePath: string,
  targetPath: string,
  format: string,
): Promise<SchematicExportResult> {
  return toolsManager<SchematicExportResult>(TOOLS_ACTIONS.SCHEMATIC_EXPORT, {
    sourcePath,
    targetPath,
    format,
  })
}
