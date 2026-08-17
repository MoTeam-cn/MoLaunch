/**
 * 工具模块 API - 启动器数据导入
 *
 * 探测本机其他启动器（PCL2/HMCL/MultiMC/CurseForge 等）的实例，
 * 将实例数据迁移到 MoLaunch（复制或符号链接）。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

/** 启动器来源类型 */
export type LauncherKind =
  | 'pcl2'
  | 'pcl2_ce'
  | 'hmcl'
  | 'multi_mc'
  | 'prism'
  | 'curseforge'
  | 'generic'

/** 可导入实例 */
export interface ImportableInstance {
  /** 实例名 */
  name: string
  /** 源实例路径 */
  path: string
  /** 检测到的 Minecraft 版本（可能为空） */
  mc_version: string | null
  /** 检测到的加载器（forge/fabric/neoforge/optifine/liteloader/quilt） */
  loader: string | null
  /** 加载器版本 */
  loader_version: string | null
}

/** 启动器来源及其实例列表 */
export interface LauncherSource {
  kind: LauncherKind
  /** UI 展示名 */
  label: string
  /** 启动器根路径 */
  base_path: string
  instances: ImportableInstance[]
}

/** 单个实例导入请求 */
export interface LauncherImportRequest {
  kind: LauncherKind
  /** 源实例路径 */
  source_path: string
  /** 导入后的实例名（缺省取源实例目录名） */
  instance_name?: string
  /** true=符号链接（共享数据），false=复制 */
  symlink: boolean
}

/** 单个实例导入结果 */
export interface ImportResultItem {
  name: string
  success: boolean
  message: string
  mc_version: string | null
  loader: string | null
}

/** 探测本机所有支持的启动器实例 */
export async function listLauncherSources(): Promise<LauncherSource[]> {
  return toolsManager<LauncherSource[]>(TOOLS_ACTIONS.LAUNCHER_IMPORT_SOURCES)
}

/** 扫描用户手动选择的路径（Generic 来源） */
export async function scanGenericPath(path: string): Promise<LauncherSource> {
  return toolsManager<LauncherSource>(TOOLS_ACTIONS.LAUNCHER_IMPORT_SCAN_PATH, {
    path,
  })
}

/** 执行单个实例导入 */
export async function runLauncherImport(
  req: LauncherImportRequest,
): Promise<ImportResultItem> {
  return toolsManager<ImportResultItem>(TOOLS_ACTIONS.LAUNCHER_IMPORT_RUN, req)
}