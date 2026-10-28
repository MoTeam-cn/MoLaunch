/**
 * 下载管理页状态恢复
 *
 * 页面进入、或前端 store 的「下载中」被提前熄灭时，向后台拉一次真实进度并写回
 * versionStore，返回后台是否仍有任务在跑。
 *
 * 用于挡住「本体下载结束 → 加载器安装仍在跑」这段空窗：
 * 此时后端 is_active 仍为 true，前端不应把页面当成空闲退回上一页。
 */

import { useVersionStore } from '@/stores/version'
import { getDownloadProgress, isDownloading } from '@/utils/api/system'
import { safeCall } from '@/utils/async'
import { toProgressSnapshot } from '@/utils/downloadProgress'

/**
 * 从后端恢复下载进度到 store
 *
 * @returns 后台是否仍有任务（true = 有，调用方应留在页面上）
 */
export async function restoreDownloadFromBackend(): Promise<boolean> {
  const versionStore = useVersionStore()

  const active = await safeCall(() => isDownloading(), 'check download state')
  if (active !== true) return false

  const raw = await safeCall(() => getDownloadProgress(), 'get download progress')
  if (!raw) {
    // 后端在跑但进度查不到：仍标记为下载中，避免页面被判定为空闲而退回
    versionStore.startDownload(versionStore.downloadingVersion || '')
    return true
  }

  versionStore.startDownload(raw.version_name || versionStore.downloadingVersion || '')
  if (!raw.stages || raw.stages.length === 0) return true

  versionStore.updateProgress(toProgressSnapshot(raw))
  return true
}
