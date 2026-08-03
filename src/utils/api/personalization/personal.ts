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