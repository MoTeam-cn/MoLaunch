/**
 * 工具模块 - 存档管理（列表 / 备份 / 恢复 / 提取种子）
 *
 * 对应后端 `tools_manager` 的 archive_list / archive_backup / archive_restore /
 * extract_save_seed action。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

/** 存档条目 */
export interface ArchiveItem {
  name: string
  path: string
  size: number
  modified: number
  has_level_dat: boolean
}

/** 存档列表结果 */
export interface ArchiveListResult {
  items: ArchiveItem[]
  total_size: number
}

/** 存档备份结果 */
export interface ArchiveBackupResult {
  success: boolean
  file_path: string
  file_size: number
}

/** 存档恢复结果 */
export interface ArchiveRestoreResult {
  success: boolean
  world_name: string
  message: string
}

/** 列出存档（可选 version_id 按版本隔离目录扫描） */
export function archiveList(versionId?: string): Promise<ArchiveListResult> {
  return toolsManager<ArchiveListResult>(TOOLS_ACTIONS.ARCHIVE_LIST, {
    version_id: versionId ?? null,
  })
}

/** 备份存档（exclude_player_data=true 为导出分享包） */
export function archiveBackup(worldName: string, outputPath: string, excludePlayerData: boolean, versionId?: string): Promise<ArchiveBackupResult> {
  return toolsManager<ArchiveBackupResult>(TOOLS_ACTIONS.ARCHIVE_BACKUP, {
    world_name: worldName,
    output_path: outputPath,
    exclude_player_data: excludePlayerData,
    version_id: versionId ?? null,
  })
}

/** 从 zip 恢复存档 */
export function archiveRestore(zipPath: string, worldName: string, versionId?: string): Promise<ArchiveRestoreResult> {
  return toolsManager<ArchiveRestoreResult>(TOOLS_ACTIONS.ARCHIVE_RESTORE, {
    zip_path: zipPath,
    world_name: worldName,
    version_id: versionId ?? null,
  })
}

/** 提取存档种子结果 */
export interface ExtractSaveSeedResult {
  /** 种子（十进制字符串，i64 范围） */
  seed: string
  /** 种子来源字段名（WorldGenSettings.seed 或 RandomSeed） */
  source: string
}

/**
 * 从存档 level.dat 提取种子
 *
 * 用于种子地图工具"从存档加载"功能：解析 level.dat 的 WorldGenSettings.seed
 * （1.16+）或 RandomSeed（1.15 及更早），返回十进制字符串避免 JS 精度丢失。
 */
export function extractSaveSeed(worldName: string, versionId?: string): Promise<ExtractSaveSeedResult> {
  return toolsManager<ExtractSaveSeedResult>(TOOLS_ACTIONS.EXTRACT_SAVE_SEED, {
    world_name: worldName,
    version_id: versionId ?? null,
  })
}
