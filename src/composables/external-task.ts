/**
 * 外部下载任务 composable
 *
 * 从 versionStore 派生下载状态与进度，以及暂停 / 取消操作。
 */
import { computed } from 'vue'
import { useVersionStore } from '@/stores/version'
import { toastError, toastInfo } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { formatBytes } from '@/utils/format'
import { pauseDownload, resumeDownload, cancelDownload } from '@/utils/api/system'

export function useExternalTask() {
  const versionStore = useVersionStore()

  // ==================== 下载状态（复用 versionStore） ====================
  const downloading = computed(() => versionStore.downloading)
  const downloadProgress = computed(() => versionStore.downloadProgress)
  const isPaused = computed(() => downloadProgress.value?.isPaused ?? false)

  const percentage = computed(() => downloadProgress.value?.percentage ?? 0)
  const speedFormatted = computed(() => formatBytes(downloadProgress.value?.global_speed ?? 0) + '/s')
  const downloadedFormatted = computed(() => {
    const d = downloadProgress.value?.global_bytes_downloaded ?? 0
    const t = downloadProgress.value?.global_bytes_total ?? 0
    return `${formatBytes(d)} / ${formatBytes(t)}`
  })

  // ==================== 下载操作 ====================
  async function togglePause() {
    try {
      if (isPaused.value) {
        await resumeDownload()
        toastInfo('下载已恢复')
      } else {
        await pauseDownload()
        toastInfo('下载已暂停')
      }
    } catch (e) {
      toastError(`操作失败: ${e instanceof Error ? e.message : String(e)}`)
    }
  }

  async function cancelDownloadTask() {
    showConfirm(
      '取消下载',
      '确定要取消当前下载任务吗？已下载的部分文件将被删除。',
      async () => {
        try {
          await cancelDownload()
          toastInfo('下载已取消')
        } catch (e) {
          toastError(`取消失败: ${e instanceof Error ? e.message : String(e)}`)
        }
      },
    )
  }

  return {
    downloading,
    downloadProgress,
    isPaused,
    percentage,
    speedFormatted,
    downloadedFormatted,
    togglePause,
    cancelDownloadTask,
  }
}