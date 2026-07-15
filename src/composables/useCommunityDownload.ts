/**
 * 社区资源下载进度管理
 * 监听后端 community-download-progress 事件
 * 参考 useDownloadPolling 的模式，但用事件推送而非轮询
 */
import { ref, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'

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

  let unlisten: (() => void) | null = null

  async function startListener() {
    if (unlisten) return
    unlisten = await listen<CommunityDownloadProgress>(
      'community-download-progress',
      (event) => {
        progress.value = event.payload
        if (event.payload.completed || event.payload.error) {
          downloading.value = false
        }
      },
    )
  }

  function stopListener() {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  function startDownload() {
    downloading.value = true
    progress.value = null
    startListener()
  }

  function stopDownload() {
    downloading.value = false
    progress.value = null
  }

  onUnmounted(() => stopListener())

  return { downloading, progress, startDownload, stopDownload, startListener, stopListener }
}
