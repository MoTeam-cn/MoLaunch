/**
 * 工具模块 - 数据类工具（崩溃分析 / 截图 / 资源包 / 版本 JSON / NBT）
 *
 * 对应后端 `tools_manager` 的 crash_analyze / screenshot_* /
 * resourcepack_* / version_json_* / nbt_parse action。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

// ==================== 崩溃日志分析 ====================

/** 单个崩溃分析条目 */
export interface CrashAnalysisItem {
  /** 分类：java_version / missing_mod / memory / driver / mod_conflict / other */
  category: string
  /** 严重级别：error / warning / info */
  severity: string
  /** 标题 */
  title: string
  /** 匹配到的相关行片段 */
  detail: string
  /** 中文修复建议 */
  suggestion: string
}

/** 崩溃日志分析结果 */
export interface CrashAnalyzeResult {
  analyses: CrashAnalysisItem[]
}

/** 崩溃日志分析 */
export function crashAnalyze(logText: string): Promise<CrashAnalyzeResult> {
  return toolsManager<CrashAnalyzeResult>(TOOLS_ACTIONS.CRASH_ANALYZE, { log_text: logText })
}

// ==================== 截图批量管理 ====================

/** 单个截图条目 */
export interface ScreenshotItem {
  path: string
  name: string
  size: number
  /** 修改时间（Unix 秒级时间戳） */
  modified: number
}

/** 截图列表结果 */
export interface ScreenshotListResult {
  items: ScreenshotItem[]
  total_size: number
}

/** 截图删除失败项 */
export interface ScreenshotFailedItem {
  path: string
  error: string
}

/** 截图删除结果 */
export interface ScreenshotDeleteResult {
  deleted_count: number
  freed_bytes: number
  failed: ScreenshotFailedItem[]
}

/** 列出截图（可选 version_id 按版本隔离目录扫描） */
export function screenshotList(versionId?: string): Promise<ScreenshotListResult> {
  return toolsManager<ScreenshotListResult>(TOOLS_ACTIONS.SCREENSHOT_LIST, {
    version_id: versionId ?? null,
  })
}

/** 批量删除截图（versionId 应与 list 时一致，用于路径校验） */
export function screenshotDelete(paths: string[], versionId?: string): Promise<ScreenshotDeleteResult> {
  return toolsManager<ScreenshotDeleteResult>(TOOLS_ACTIONS.SCREENSHOT_DELETE, {
    paths,
    version_id: versionId ?? null,
  })
}

// ==================== 资源包转换 ====================

/** 单个资源包条目 */
export interface ResourcePackItem {
  name: string
  path: string
  /** 格式：zip / folder */
  format: string
  size: number
}

/** 资源包列表结果 */
export interface ResourcePackListResult {
  items: ResourcePackItem[]
}

/** 资源包转换结果 */
export interface ResourcePackConvertResult {
  success: boolean
  output_path: string
  message: string
}

/** 列出资源包（可选 version_id 按版本隔离目录扫描） */
export function resourcepackList(versionId?: string): Promise<ResourcePackListResult> {
  return toolsManager<ResourcePackListResult>(TOOLS_ACTIONS.RESOURCEPACK_LIST, {
    version_id: versionId ?? null,
  })
}

/** 转换资源包格式（zip ↔ folder，可选 versionId 按版本隔离目录校验路径） */
export function resourcepackConvert(
  path: string,
  targetFormat: 'zip' | 'folder',
  versionId?: string,
): Promise<ResourcePackConvertResult> {
  return toolsManager<ResourcePackConvertResult>(TOOLS_ACTIONS.RESOURCEPACK_CONVERT, {
    path,
    target_format: targetFormat,
    version_id: versionId ?? null,
  })
}

// ==================== 版本 JSON 编辑 ====================

/** 版本 JSON 读取结果 */
export interface VersionJsonReadResult {
  content: string
  path: string
}

/** 版本 JSON 保存结果 */
export interface VersionJsonSaveResult {
  success: boolean
}

/** 读取版本 JSON */
export function versionJsonRead(versionId: string): Promise<VersionJsonReadResult> {
  return toolsManager<VersionJsonReadResult>(TOOLS_ACTIONS.VERSION_JSON_READ, { version_id: versionId })
}

/** 保存版本 JSON（后端会先校验 JSON 合法性） */
export function versionJsonSave(versionId: string, content: string): Promise<VersionJsonSaveResult> {
  return toolsManager<VersionJsonSaveResult>(TOOLS_ACTIONS.VERSION_JSON_SAVE, { version_id: versionId, content })
}

// ==================== NBT 数据查看 ====================

/** NBT 树节点 */
export interface NbtNode {
  name: string
  tag_type: string
  value: unknown | null
  children: NbtNode[]
}

/** NBT 解析结果 */
export interface NbtParseResult {
  root: NbtNode
}

/** 解析 NBT 文件 */
export function nbtParse(filePath: string): Promise<NbtParseResult> {
  return toolsManager<NbtParseResult>(TOOLS_ACTIONS.NBT_PARSE, { file_path: filePath })
}
