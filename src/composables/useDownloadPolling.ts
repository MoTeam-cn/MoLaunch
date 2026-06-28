import { ref, onUnmounted } from 'vue'
import { getDownloadProgress, isDownloading as checkIsDownloading } from '@/utils/tauri'

export function useDownloadPolling(interval = 300) {
  const isDownloading = ref(false)
  const downloadProgress = ref<any>(null)
  const downloadStage = ref('')
  const downloadPercentage = ref(0)
  const downloadedVersions = ref<string[]>([])

  let pollTimer: ReturnType<typeof setInterval> | null = null

  function startPolling() {
    if (pollTimer) return
    isDownloading.value = true

    pollTimer = setInterval(async () => {
      try {
        const [progress, downloading] = await Promise.all([
          getDownloadProgress(),
          checkIsDownloading()
        ])

        downloadProgress.value = progress
        isDownloading.value = downloading

        if (progress) {
          downloadStage.value = progress.stage === 0 ? 'Patching' :
                               progress.stage === 1 ? 'Downloading' : 'Verifying'
          downloadPercentage.value = progress.total > 0
            ? Math.round((progress.current / progress.total) * 100)
            : 0
        }

        if (!downloading) {
          stopPolling()
        }
      } catch (e) {
        console.error('Polling error:', e)
      }
    }, interval)
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
    isDownloading.value = false
  }

  onUnmounted(() => {
    stopPolling()
  })

  return {
    isDownloading,
    downloadProgress,
    downloadStage,
    downloadPercentage,
    downloadedVersions,
    startPolling,
    stopPolling
  }
}
