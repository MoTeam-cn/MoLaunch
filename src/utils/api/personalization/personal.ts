import { VERSION_LIST_ACTIONS, versionListManager } from '../version-list-manager'
import type { PersonalizationUpdate, VersionPersonalization } from './types'

/**
 * 获取版本个性化设置
 */
export async function getVersionPersonalization(versionId: string): Promise<VersionPersonalization> {
  return versionListManager<VersionPersonalization>(VERSION_LIST_ACTIONS.GET_VERSION_PERSONALIZATION, { versionId })
}

/**
 * 更新版本个性化字段（传 undefined 表示不修改该字段）
 */
export async function updateVersionPersonalization(
  versionId: string,
  update: PersonalizationUpdate,
): Promise<void> {
  return versionListManager<void>(VERSION_LIST_ACTIONS.UPDATE_VERSION_PERSONALIZATION, { versionId, update })
}

/**
 * 补全版本文件（校验并下载缺失的 libraries/assets）
 */
export async function fixVersionFiles(versionId: string): Promise<void> {
  return versionListManager<void>(VERSION_LIST_ACTIONS.FIX_VERSION_FILES, { versionId })
}

/** 加载器检测/重装结果 */
export interface RepairLoaderResult {
  loaderType: string | null
  loaderVersion: string
  mcVersion: string
  damaged: boolean
  repaired: boolean
  message: string
}

/** 加载器修复进度阶段 */
export type RepairLoaderPhase = 'scanning' | 'installing' | 'merging' | 'done' | 'error'

/** 加载器修复进度事件负载（与后端 RepairProgress 对应） */
export interface RepairLoaderProgress {
  versionId: string
  phase: RepairLoaderPhase
  progress: number
  damaged: boolean
  repaired: boolean
  loaderType: string | null
  loaderVersion: string
  mcVersion: string
  message: string
}

/** 加载器修复进度事件名（与后端 REPAIR_LOADER_PROGRESS_EVENT 对应） */
export const REPAIR_LOADER_PROGRESS_EVENT = 'repair-loader-progress'

/**
 * 检测 Forge/Fabric/LiteLoader 是否损坏，损坏则自动重新安装
 *
 * 执行进度通过 `REPAIR_LOADER_PROGRESS_EVENT` 事件推送，本函数返回最终结果。
 */
export async function repairVersionLoader(versionId: string): Promise<RepairLoaderResult> {
  return versionListManager<RepairLoaderResult>(VERSION_LIST_ACTIONS.REPAIR_VERSION_LOADER, { versionId })
}