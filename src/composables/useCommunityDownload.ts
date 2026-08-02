/**
 * 社区资源下载进度管理
 * 监听后端 community-download-progress 事件
 * 参考 useDownloadPolling 的模式，但用事件推送而非轮询
 */
import { ref } from 'vue'
import { useTauriEvent } from '@/composables/useTauriEvent'

export interface CommunityDownloadProgress {
  fileName: string
  downloaded: number
  total: number
  speed: number
  completed: boolean
  error: string | null
}

export function useCommunityDownload() {
  const downloading = ref(false)
  const progress = ref<CommunityDownloadProgress | null>(null)

  const { start, stop } = useTauriEvent<CommunityDownloadProgress>(
    'community-download-progress',
    (payload) => {
      progress.value = payload
      if (payload.completed || payload.error) {
        downloading.value = false
      }
    },
  )

  function startDownload() {
    downloading.value = true
    progress.value = null
    start()
  }

  function stopDownload() {
    downloading.value = false
    progress.value = null
  }

  return {
    downloading,
    progress,
    startDownload,
    stopDownload,
    // 兼容旧调用方：startListener/stopListener 指向 useTauriEvent 的 start/stop
    startListener: start,
    stopListener: stop,
  }
}
