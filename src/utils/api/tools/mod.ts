/**
 * 工具模块 - Mod 依赖检测 + Mod 去重扫描
 *
 * 对应后端 `tools_manager` 的 mod_dependency_check / mod_dedup_scan action。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

/** 缺失的依赖项 */
export interface MissingDep {
  /** 依赖此 mod 的文件名 */
  required_by: string
  /** 缺失的 mod_id */
  mod_id: string
}

/** 冲突依赖项（未来扩展用） */
export interface ConflictDep {
  mod_id: string
  reason: string
}

/** Mod 依赖检测结果 */
export interface ModDependencyResult {
  /** 依赖的 mod_id 不在已安装列表中 */
  missing: MissingDep[]
  /** 冲突依赖（暂时留空，未来扩展） */
  conflicts: ConflictDep[]
}

/** Mod 依赖检测 */
export function modDependencyCheck(versionId: string): Promise<ModDependencyResult> {
  return toolsManager<ModDependencyResult>(TOOLS_ACTIONS.MOD_DEPENDENCY_CHECK, { version_id: versionId })
}

/** 重复 Mod 的单个版本条目 */
export interface DuplicateVersion {
  version: string
  file_name: string
  file_size: number
}

/** 重复的 Mod（同一 mod_id 有多个版本） */
export interface DuplicateMod {
  mod_id: string
  versions: DuplicateVersion[]
}

/** Mod 去重扫描结果 */
export interface ModDedupResult {
  duplicates: DuplicateMod[]
}

/** Mod 去重扫描 */
export function modDedupScan(versionId: string): Promise<ModDedupResult> {
  return toolsManager<ModDedupResult>(TOOLS_ACTIONS.MOD_DEDUP_SCAN, { version_id: versionId })
}
