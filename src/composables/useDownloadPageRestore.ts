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
import type { DownloadStage, RawDownloadStage } from '@/types/download'

/** 后端 stages → 前端 stages */
function mapStages(raw: RawDownloadStage[]): DownloadStage[] {
  return raw.map((s) => ({
    name: s.name,
    progress: s.progress,
    weight: s.weight,
    status: s.status,
    bytes_downloaded: s.bytes_downloaded,
    bytes_total: s.bytes_total,
    files_downloaded: s.files_downloaded || 0,
    files_total: s.files_total || 0,
    group: s.group ?? null,
  }))
}

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

  const stages = mapStages(raw.stages)
  let weightedProgress = 0
  let totalWeight = 0
  for (const s of stages) {
    totalWeight += s.weight
    weightedProgress += s.progress * s.weight
  }
  const percentage = totalWeight > 0
    ? Math.min(100, parseFloat(((weightedProgress / totalWeight) * 100).toFixed(1)))
    : 0

  versionStore.updateProgress({
    stages,
    current_stage_index: raw.current_stage_index ?? 0,
    global_speed: raw.global_speed ?? 0,
    global_bytes_downloaded: raw.global_bytes_downloaded ?? 0,
    global_bytes_total: raw.global_bytes_total ?? 0,
    percentage,
    isPaused: raw.stages.some((s) => s.is_paused === true),
  })
  return true
}
