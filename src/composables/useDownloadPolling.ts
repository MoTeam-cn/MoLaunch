import { watch } from 'vue'
import { useVersionStore } from '@/stores/version'
import { getDownloadProgress } from '@/utils/tauri'
import type { DownloadStage } from '@/stores/version'

let pollTimer: ReturnType<typeof setInterval> | null = null
let pollCount = 0

function startPolling(versionStore: ReturnType<typeof useVersionStore>) {
  if (pollTimer) return

  pollCount = 0
  console.log('[Polling] Starting download polling...')

  pollTimer = setInterval(async () => {
    pollCount++
    try {
      const progress = await getDownloadProgress()

      if (pollCount % 10 === 0) {
        console.log(`[Polling] #${pollCount} progress=`, progress)
      }

      if (progress && progress.stages && progress.stages.length > 0) {
        const stages: DownloadStage[] = progress.stages.map((s: any) => ({
          name: s.name,
          progress: s.progress,
          weight: s.weight,
          status: s.status,
          bytes_downloaded: s.bytes_downloaded,
          bytes_total: s.bytes_total,
          files_downloaded: s.files_downloaded || 0,
          files_total: s.files_total || 0,
        }))

        let weightedProgress = 0
        let totalWeight = 0
        for (const stage of stages) {
          totalWeight += stage.weight
          weightedProgress += stage.progress * stage.weight
        }
        const percentage = totalWeight > 0
          ? Math.min(100, Math.round((weightedProgress / totalWeight) * 100))
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
          console.log('[Polling] Download complete, stopping polling')
          setTimeout(() => {
            stopPolling()
            versionStore.finishDownload()
          }, 1500)
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
