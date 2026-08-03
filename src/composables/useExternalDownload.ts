/**
 * 外部下载工具的 composable
 *
 * 编排来源管理（external-source）与任务状态（external-task），对外签名保持不变。
 */
import { computed, watch, onMounted } from 'vue'
import { useVersionStore } from '@/stores/version'
import { toastError, toastInfo } from '@/utils/toast'
import { showModal } from '@/utils/modal'
import { isCancelledError } from '@/utils/async'
import { downloadFile } from '@/utils/api/tools'
import { useExternalSource } from './external-source'
import { useExternalTask } from './external-task'

export function useExternalDownload() {
  const versionStore = useVersionStore()
  const source = useExternalSource()
  const task = useExternalTask()

  const currentFileName = computed(() => versionStore.downloadingVersion ?? source.fileName.value)

  const canStartDownload = computed(() => {
    return source.url.value.trim() && source.fileName.value.trim() && !task.downloading.value
  })

  // ==================== 下载逻辑 ====================
  async function startDownload() {
    const urlVal = source.url.value.trim()
    const nameVal = source.fileName.value.trim()

    if (!urlVal) {
      toastError('请输入下载地址')
      return
    }
    if (!nameVal) {
      toastError('请输入文件名')
      return
    }
    if (!source.isValidUrl(urlVal)) {
      toastError('下载地址必须以 http:// 或 https:// 开头')
      return
    }

    versionStore.startDownload(nameVal)
    toastInfo(`开始下载: ${nameVal}`)

    downloadFile(urlVal, nameVal)
      .then(async () => {
        await source.refreshFiles()
        source.url.value = ''
        source.fileName.value = ''
      })
      .catch((e) => {
        const msg = e instanceof Error ? e.message : String(e)
        // 用户主动取消：仅 toast 提示，不弹错误窗
        if (isCancelledError(e)) {
          toastInfo('下载已取消')
          versionStore.finishDownload()
          return
        }
        // 真实失败：后端已 mark_failed 重置 is_active，用 showModal 让用户确认后退出下载页
        showModal({
          type: 'error',
          title: '下载失败',
          message: msg,
          onConfirm: () => {
            versionStore.finishDownload()
          },
        })
      })
  }

  // ==================== 监听下载完成（刷新文件列表） ====================
  watch(task.downloading, (isDownloading, wasDownloading) => {
    if (wasDownloading && !isDownloading) {
      source.refreshFiles()
    }
  })

  // ==================== 生命周期 ====================
  onMounted(async () => {
    await Promise.all([source.loadCustomDir(), source.loadDownloadDir(), source.refreshFiles()])
  })

  return {
    // 表单状态
    url: source.url,
    fileName: source.fileName,
    isFetchingFilename: source.isFetchingFilename,
    onFileNameInput: source.onFileNameInput,
    // 下载目录
    downloadDir: source.downloadDir,
    isCustomDir: source.isCustomDir,
    selectDownloadDir: source.selectDownloadDir,
    resetDownloadDir: source.resetDownloadDir,
    openDownloadDir: source.openDownloadDir,
    // 下载状态
    downloading: task.downloading,
    isPaused: task.isPaused,
    percentage: task.percentage,
    speedFormatted: task.speedFormatted,
    downloadedFormatted: task.downloadedFormatted,
    currentFileName,
    canStartDownload,
    startDownload,
    togglePause: task.togglePause,
    cancelDownloadTask: task.cancelDownloadTask,
    // 文件列表
    files: source.files,
    deleteFile: source.deleteFile,
  }
}