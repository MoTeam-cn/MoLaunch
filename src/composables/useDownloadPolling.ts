import { watch } from 'vue'
import { useVersionStore } from '@/stores/version'
import { getDownloadProgress } from '@/utils/tauri'
import { toastSuccess } from '@/utils/toast'
import type { DownloadStage, RawDownloadStage } from '@/types/download'
import { safeCall } from '@/utils/async'

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
    await safeCall(async () => {
      const progress = await getDownloadProgress()

      if (import.meta.env.DEV && pollCount % 10 === 0) {
        console.debug(`[Polling] #${pollCount} progress=`, progress)
      }

      if (progress && progress.stages && progress.stages.length > 0) {
        // 先检查错误码，避免在失败状态下继续轮询
        // 注意：这里只 stopPolling，不 finishDownload / toastError。
        // 失败的 UI 提示和 finishDownload 由调用方的 catch 统一处理
        // （showModal + onConfirm: finishDownload），避免轮询抢先 finishDownload
        // 导致调用方 showModal 还没显示用户就被 router.back() 带走。
        if (progress.error_code && progress.error_code !== 0) {
          if (import.meta.env.DEV) {
            console.debug('[Polling] Download failed with error_code=', progress.error_code, ' (交给调用方 catch 处理)')
          }
          stopPolling()
          return
        }

        // 检测进度回退：downloaded 或 total 突然变小（非初始化阶段）
        // 这是定位「timeout 后进度归零」bug 的关键日志
        const newDownloaded = progress.global_bytes_downloaded ?? 0
        const newTotal = progress.global_bytes_total ?? 0
        if (import.meta.env.DEV && pollCount > 2) {
          if (newDownloaded < lastGlobalDownloaded) {
            console.debug(
              `[Polling] [WARN] downloaded 回退! ${lastGlobalDownloaded} -> ${newDownloaded} (差值 ${lastGlobalDownloaded - newDownloaded}, poll #${pollCount})`,
              JSON.parse(JSON.stringify(progress))
            )
          }
          if (newTotal < lastGlobalTotal && newTotal > 0) {
            console.debug(
              `[Polling] [WARN] total 回退! ${lastGlobalTotal} -> ${newTotal} (差值 ${lastGlobalTotal - newTotal}, poll #${pollCount})`,
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

        // 检测暂停状态：任意 stage 携带 is_paused=true 即表示全局暂停
        const isPaused = progress.stages.some((s: RawDownloadStage) => s.is_paused === true)

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
          isPaused,
        })

        if (progress.is_complete) {
          if (import.meta.env.DEV) {
            console.debug('[Polling] Download complete, stopping polling')
          }
          stopPolling()
          const completedName = versionStore.downloadingVersion || '下载任务'
          versionStore.finishDownload()
          toastSuccess(`${completedName} 下载完成`)
          return
        }
      }
    }, '[Polling] poll download progress')
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
