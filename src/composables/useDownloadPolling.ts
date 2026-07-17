import { watch } from 'vue'
import { useVersionStore } from '@/stores/version'
import { getDownloadProgress } from '@/utils/tauri'
import type { DownloadStage, RawDownloadStage } from '@/types/download'

let pollTimer: ReturnType<typeof setInterval> | null = null
let pollCount = 0
// 上一次轮询的字节数，用于检测进度回退（突然归零）的 bug
let lastGlobalDownloaded = 0
let lastGlobalTotal = 0

function startPolling(versionStore: ReturnType<typeof useVersionStore>) {
  if (pollTimer) return

  pollCount = 0
  lastGlobalDownloaded = 0
  lastGlobalTotal = 0
  if (import.meta.env.DEV) {
    console.debug('[Polling] Starting download polling...')
  }

  pollTimer = setInterval(async () => {
    pollCount++
    try {
      const progress = await getDownloadProgress()

      if (import.meta.env.DEV && pollCount % 10 === 0) {
        console.debug(`[Polling] #${pollCount} progress=`, progress)
      }

      if (progress && progress.stages && progress.stages.length > 0) {
        // 先检查错误码，避免在失败状态下继续轮询
        if (progress.error_code && progress.error_code !== 0) {
          if (import.meta.env.DEV) {
            console.debug('[Polling] Download failed with error_code=', progress.error_code)
          }
          stopPolling()
          versionStore.finishDownload()
          return
        }

        // 检测进度回退：downloaded 或 total 突然变小（非初始化阶段）
        // 这是定位「timeout 后进度归零」bug 的关键日志
        const newDownloaded = progress.global_bytes_downloaded ?? 0
        const newTotal = progress.global_bytes_total ?? 0
        if (import.meta.env.DEV && pollCount > 2) {
          if (newDownloaded < lastGlobalDownloaded) {
            console.debug(
              `[Polling] ⚠️ downloaded 回退! ${lastGlobalDownloaded} -> ${newDownloaded} (差值 ${lastGlobalDownloaded - newDownloaded}, poll #${pollCount})`,
              JSON.parse(JSON.stringify(progress))
            )
          }
          if (newTotal < lastGlobalTotal && newTotal > 0) {
            console.debug(
              `[Polling] ⚠️ total 回退! ${lastGlobalTotal} -> ${newTotal} (差值 ${lastGlobalTotal - newTotal}, poll #${pollCount})`,
              JSON.parse(JSON.stringify(progress))
            )
          }
        }
        if (newDownloaded > 0) lastGlobalDownloaded = newDownloaded
        if (newTotal > 0) lastGlobalTotal = newTotal

        const stages: DownloadStage[] = progress.stages.map((s: RawDownloadStage) => ({
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

        let weightedProgress = 0
        let totalWeight = 0
        for (const stage of stages) {
          totalWeight += stage.weight
          weightedProgress += stage.progress * stage.weight
        }
        const percentage = totalWeight > 0
          ? Math.min(100, parseFloat(((weightedProgress / totalWeight) * 100).toFixed(1)))
          : 0

        versionStore.updateProgress({
          stages,
          current_stage_index: progress.current_stage_index ?? 0,
          global_speed: progress.global_speed ?? 0,
          global_bytes_downloaded: progress.global_bytes_downloaded ?? 0,
          global_bytes_total: progress.global_bytes_total ?? 0,
          percentage,
        })

        if (progress.is_complete) {
          if (import.meta.env.DEV) {
            console.debug('[Polling] Download complete, stopping polling')
          }
          stopPolling()
          versionStore.finishDownload()
          return
        }
      }
    } catch (e) {
      console.error('[Polling] Error:', e)
    }
  }, 300)
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
  pollCount = 0
  lastGlobalDownloaded = 0
  lastGlobalTotal = 0
}

export function initDownloadPolling() {
  const versionStore = useVersionStore()

  watch(
    () => versionStore.downloading,
    (isDownloading) => {
      if (isDownloading) {
        startPolling(versionStore)
      } else {
        stopPolling()
      }
    }
  )
}
